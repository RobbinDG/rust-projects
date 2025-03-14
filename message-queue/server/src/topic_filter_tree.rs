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

    fn get_clients(&self, filter_by_level: &Vec<TopicLiteral>, depth: usize) -> HashSet<&ClientID>;
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

    fn get_clients(&self, _: &Vec<TopicLiteral>, _: usize) -> HashSet<&ClientID> {
        self.iter().collect::<HashSet<&ClientID>>()
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

    fn get_clients(&self, filter_by_level: &Vec<TopicLiteral>, depth: usize) -> HashSet<&ClientID> {
        let filter = filter_by_level
            .get(depth)
            .unwrap_or(&TopicLiteral::Wildcard);

        let mut elements = self.terminating.iter().collect::<HashSet<&ClientID>>();
        match filter {
            TopicLiteral::Name(name) => {
                if let Some(sub) = self.sub.get(name) {
                    elements.extend(sub.get_clients(filter_by_level, depth + 1));
                }
            }
            TopicLiteral::Wildcard => {
                for sub in self.sub.values() {
                    elements.extend(sub.get_clients(filter_by_level, depth + 1));
                }
            }
        };
        elements
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
    /// returns: `HashSet<&ClientID, Global>` a reference to all client ID's.
    pub fn get_clients(&self, address: (String, String)) -> HashSet<&ClientID> {
        self.topics.get_clients(
            &vec![TopicLiteral::Name(address.0), TopicLiteral::Name(address.1)],
            0,
        )
    }
}
