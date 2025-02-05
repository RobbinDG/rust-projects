use backend::protocol::client_id::ClientID;
use backend::protocol::queue_id::TopicLiteral;
use std::collections::{HashMap, HashSet};

trait ClientCollection {
    fn insert_client(
        &mut self,
        client_id: ClientID,
        filter_by_level: &Vec<TopicLiteral>,
        depth: usize,
    ) -> bool;

    fn remove_client(
        &mut self,
        client_id: &ClientID,
        filter_by_level: &Vec<TopicLiteral>,
        depth: usize,
    ) -> bool;
}

type TopicFilterTreeLeaf = HashSet<ClientID>;

impl ClientCollection for TopicFilterTreeLeaf {
    fn insert_client(&mut self, client_id: ClientID, _: &Vec<TopicLiteral>, _: usize) -> bool {
        self.insert(client_id);
        true
    }

    fn remove_client(&mut self, client_id: &ClientID, _: &Vec<TopicLiteral>, _: usize) -> bool {
        self.remove(&client_id);
        true
    }
}

pub struct TopicFilterTreeNode<T: ClientCollection> {
    pub terminating: TopicFilterTreeLeaf,
    pub sub: HashMap<String, T>,
}

impl<T: ClientCollection> TopicFilterTreeNode<T> {
    pub fn new() -> Self {
        Self {
            terminating: HashSet::new(),
            sub: HashMap::new(),
        }
    }
}

impl<T: ClientCollection> ClientCollection for TopicFilterTreeNode<T> {
    fn insert_client(
        &mut self,
        client_id: ClientID,
        filter_by_level: &Vec<TopicLiteral>,
        depth: usize,
    ) -> bool {
        let filter = filter_by_level
            .get(depth)
            .unwrap_or(&TopicLiteral::Wildcard);

        match filter {
            TopicLiteral::Name(name) => match self.sub.get_mut(name) {
                Some(sub) => sub.insert_client(client_id, filter_by_level, depth + 1),
                None => false,
            },
            TopicLiteral::Wildcard => {
                for sub in self.sub.values_mut() {
                    sub.insert_client(client_id.clone(), filter_by_level, depth + 1);
                }
                true
            }
        }
    }

    fn remove_client(
        &mut self,
        client_id: &ClientID,
        filter_by_level: &Vec<TopicLiteral>,
        depth: usize,
    ) -> bool {
        let filter = filter_by_level
            .get(depth)
            .unwrap_or(&TopicLiteral::Wildcard);

        match filter {
            TopicLiteral::Name(name) => match self.sub.get_mut(name) {
                Some(sub) => sub.remove_client(client_id, filter_by_level, depth + 1),
                None => false,
            },
            TopicLiteral::Wildcard => {
                for sub in self.sub.values_mut() {
                    sub.remove_client(client_id, filter_by_level, depth + 1);
                }
                true
            }
        }
    }
}

/// A data structure to efficiently store subscribers under their desired filter.
pub struct TopicFilterTree {
    topics: TopicFilterTreeNode<TopicFilterTreeNode<TopicFilterTreeLeaf>>,
    filters: HashMap<ClientID, Vec<TopicLiteral>>,
}

impl TopicFilterTree {
    pub fn new() -> TopicFilterTree {
        Self {
            topics: TopicFilterTreeNode::new(),
            filters: HashMap::new(),
        }
    }

    pub fn subtopic_tree(&self) -> HashMap<&String, HashSet<&String>> {
        self.topics
            .sub
            .iter()
            .map(|(k, v)| (k, v.sub.keys().collect()))
            .collect()
    }

    pub fn is_filter_nonempty(&self, filter: (&TopicLiteral, &TopicLiteral)) -> bool {
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
    pub fn insert(&mut self, client: ClientID, filter: &Vec<TopicLiteral>) -> bool {
        self.topics.insert_client(client, &filter, 0)
    }

    pub fn remove(&mut self, client: &ClientID) -> bool {
        if let Some(filter) = self.filters.get(client) {
            self.topics.remove_client(client, filter, 0)
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
            .into_iter()
            .collect()
    }

    fn get_clients_filter_tree(
        tree: &TopicFilterTreeNode<TopicFilterTreeNode<TopicFilterTreeLeaf>>,
        address: (String, String),
    ) -> HashSet<&ClientID> {
        let mut clients = match tree.sub.get(&address.0) {
            None => HashSet::new(),
            Some(subtree) => Self::get_clients_subtree(subtree, address.1.clone()),
        };
        clients.extend(&tree.terminating);
        clients
    }

    fn get_clients_subtree(
        address: &TopicFilterTreeNode<TopicFilterTreeLeaf>,
        filter: String,
    ) -> HashSet<&ClientID> {
        let mut clients = match address.sub.get(&filter) {
            None => HashSet::new(),
            Some(c) => c.iter().collect(),
        };
        clients.extend(&address.terminating);
        clients
    }
}
