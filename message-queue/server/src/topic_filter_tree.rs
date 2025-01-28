use backend::protocol::client_id::ClientID;
use backend::protocol::queue_id::TopicLiteral;
use std::collections::{HashMap, HashSet};

type TopicFilterTreeLeaf = HashSet<ClientID>;

pub struct TopicFilterTreeNode<T> {
    pub terminating: TopicFilterTreeLeaf,
    pub sub: HashMap<String, T>,
}

impl<T> TopicFilterTreeNode<T> {
    pub fn new() -> Self {
        Self {
            terminating: HashSet::new(),
            sub: HashMap::new(),
        }
    }
}

/// A data structure to efficiently store subscribers under their desired filter.
pub struct TopicFilterTree {
    topics: TopicFilterTreeNode<TopicFilterTreeNode<TopicFilterTreeLeaf>>,
    filters: HashMap<ClientID, (TopicLiteral, TopicLiteral)>,
}

impl TopicFilterTree {
    pub fn new() -> TopicFilterTree {
        Self {
            topics: TopicFilterTreeNode::new(),
            filters: HashMap::new(),
        }
    }

    pub fn subtopics(&self) -> HashMap<&String, Vec<&String>> {
        self.topics
            .sub
            .iter()
            .map(|(k, v)| (k, v.sub.keys().collect()))
            .collect()
    }

    pub fn filter_nonempty(&self, filter: (&TopicLiteral, &TopicLiteral)) -> bool {
        match filter.0 {
            TopicLiteral::Name(s) => self.topics.sub.get(s).map_or(false, |sub| match filter.1 {
                TopicLiteral::Name(s) => sub.sub.contains_key(s),
                TopicLiteral::Wildcard => true,
            }),
            TopicLiteral::Wildcard => true,
        }
    }

    pub fn create_subtopic(&mut self, subtopic: String) {
        self.topics
            .sub
            .entry(subtopic)
            .or_insert_with(TopicFilterTreeNode::new);
    }

    pub fn create_subsubtopic(&mut self, subtopic: String, name: String) {
        self.topics.sub.get_mut(&subtopic).map(|st| {
            st.sub.entry(name).or_insert_with(TopicFilterTreeLeaf::new);
        });
    }

    /// Insert a client's ID into the tree given a filter. This will fail if the topic
    /// with that filter does not exist.
    ///
    /// # Arguments
    ///
    /// * `client`: the client's ID to insert.
    /// * `filter`: the filter under which they want to receive.
    ///
    /// returns: `bool` if the insertion failed due to the filter not yet existing.
    pub fn insert(&mut self, client: ClientID, filter: (TopicLiteral, TopicLiteral)) -> bool {
        let clients = self.get_client_set(filter);
        clients
            .and_then(|c| Some(c.insert(client)))
            .unwrap_or(false)
    }

    pub fn remove(&mut self, client: &ClientID) -> bool {
        if let Some(filter) = self.filters.get(client) {
            self.get_client_set(filter.clone())
                .and_then(|clients| Some(clients.remove(client)))
                .unwrap_or(false)
        } else {
            false
        }
    }

    /// Gets a vector of clients for a given topic address.
    ///
    /// # Arguments
    ///
    /// * `address`: the address to look up.
    ///
    /// returns: `Vec<&ClientID, Global>` a reference to all client ID's.
    pub fn get_clients(&self, address: (String, String)) -> Vec<&ClientID> {
        Self::get_clients_filter_tree(&self.topics, address)
    }

    fn get_client_set(
        &mut self,
        filter: (TopicLiteral, TopicLiteral),
    ) -> Option<&mut TopicFilterTreeLeaf> {
        match filter.0 {
            TopicLiteral::Name(s) => self.topics.sub.get_mut(&s).and_then(|sub| match filter.1 {
                TopicLiteral::Name(s) => sub.sub.get_mut(&s),
                TopicLiteral::Wildcard => Some(&mut sub.terminating),
            }),
            TopicLiteral::Wildcard => Some(&mut self.topics.terminating),
        }
    }

    fn get_clients_filter_tree(
        tree: &TopicFilterTreeNode<TopicFilterTreeNode<TopicFilterTreeLeaf>>,
        address: (String, String),
    ) -> Vec<&ClientID> {
        let mut clients = match tree.sub.get(&address.0) {
            None => vec![],
            Some(subtree) => Self::get_clients_subtree(subtree, address.1.clone()),
        };
        clients.extend(&tree.terminating);
        clients
    }

    fn get_clients_subtree(
        address: &TopicFilterTreeNode<TopicFilterTreeLeaf>,
        filter: String,
    ) -> Vec<&ClientID> {
        let mut clients = match address.sub.get(&filter) {
            None => vec![],
            Some(c) => c.iter().collect(),
        };
        clients.extend(&address.terminating);
        clients
    }
}
