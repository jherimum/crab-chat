use std::{collections::HashSet, error::Error, str::FromStr};

use libp2p::Swarm;
use log::info;
use serde::{Deserialize, Serialize};

use crate::{ListMode, ListRequest, RecipeBehaviour, TOPIC, recipes};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Command {
    ListPeers,
    ListLocalRecipes,
    ListAllRecipes,
    ListPeerRecipes(String),
    CreateRecipe(String, String, String),
    Unknown(String),
}

impl Command {
    pub async fn execute(
        &self,
        swarm: &mut Swarm<RecipeBehaviour>,
    ) -> Result<(), Box<dyn Error + Sync + Send + 'static>> {
        match self {
            Command::ListPeers => {
                let nodes = swarm
                    .behaviour()
                    .mdns
                    .discovered_nodes()
                    .into_iter()
                    .collect::<HashSet<_>>();
                info!("Listing all Peers:");
                for peer in nodes {
                    info!("Peer: {}", peer);
                }
                Ok(())
            }
            Command::ListLocalRecipes => {
                info!("Listing my recipes:");
                for r in recipes().await.load_recipes().unwrap() {
                    info!("{:?}", r);
                }
                Ok(())
            }
            Command::CreateRecipe(name, ingredients, instructions) => {
                let recipe = recipes()
                    .await
                    .create_new_recipe(name, ingredients, instructions)?;
                info!("Recipe created: {:?}", recipe);
                Ok(())
            }
            Command::ListAllRecipes => {
                let request = ListRequest {
                    mode: ListMode::ALL,
                };
                swarm
                    .behaviour_mut()
                    .floodsub
                    .publish(TOPIC.clone(), serde_json::to_vec(&request).unwrap());

                Ok(())
            }
            Command::ListPeerRecipes(peer_id) => {
                let request = ListRequest {
                    mode: ListMode::One(peer_id.clone()),
                };
                swarm
                    .behaviour_mut()
                    .floodsub
                    .publish(TOPIC.clone(), serde_json::to_vec(&request).unwrap());
                Ok(())
            }
            Command::Unknown(line) => {
                log::error!("Unknown command: {}", line);
                Ok(())
            }
        }
    }
}

impl FromStr for Command {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.split_once(" ") {
            Some(("ls", rest)) => {
                let tokens = rest.split_ascii_whitespace().collect::<Vec<_>>();
                match tokens.as_slice() {
                    ["p"] => Ok(Command::ListPeers),
                    ["r"] => Ok(Command::ListLocalRecipes),
                    ["r", "all"] => Ok(Command::ListAllRecipes),
                    ["r", peer_id] => Ok(Command::ListPeerRecipes(peer_id.to_string())),
                    _ => Ok(Command::Unknown(s.to_string())),
                }
            }
            Some(("create", recipe)) => {
                let mut parts = recipe.splitn(3, "|");
                let name = parts.next().unwrap_or_default();
                let ingredients = parts.next().unwrap_or_default();
                let instructions = parts.next().unwrap_or_default();
                Ok(Command::CreateRecipe(
                    name.to_string(),
                    ingredients.to_string(),
                    instructions.to_string(),
                ))
            }
            _ => Ok(Command::Unknown(s.to_string())),
        }
    }
}
