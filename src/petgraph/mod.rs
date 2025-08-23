use petgraph_lib::{stable_graph::{StableGraph, NodeIndex}};
use super::{Page, ReqwestError};

pub struct WikipediaGraph<NodeIndex = petgraph_lib::stable_graph::DefaultIx, EdgeIndex = petgraph_lib::Directed> {
    graph: StableGraph<Page, (), EdgeIndex, NodeIndex>,
}

impl<Ix: petgraph_lib::stable_graph::IndexType> WikipediaGraph<Ix> {
    pub fn new() -> Self {
        WikipediaGraph {
            graph: StableGraph::default()
        }
    }

    pub fn add_page(&mut self, page: Page) {
        self.graph.add_node(page);
    }

    pub fn expand_page(&mut self, index: NodeIndex<Ix>) -> Result<(), ReqwestError> {
        let mut weight: Page  = self.graph.node_weight(index).expect("Index doesn't exist").clone();

        let _ = weight.load_title();

        let connections = weight.get_connections()?;

        let mut graph = &mut self.graph;

        connections
            .into_iter()
            .for_each(|connection| {
                let connection_index = graph.add_node(connection);

                // Todo: Error logic like everywhere
                graph.try_add_edge(index, connection_index, ()).unwrap();
            });

        Ok(())

        
    }
}