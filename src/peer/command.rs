use crate::{
    peer::RecipeBehaviour,
    peer::message::{ListMode, ListRequest, RecipeMessage},
    recipes::Recipes,
};
use libp2p::floodsub::Topic;
use log::info;
use serde::{Deserialize, Serialize};
use std::error::Error;

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
        topic: Topic,
        recipes: &Recipes,
        behaviour: &mut RecipeBehaviour,
    ) -> Result<(), Box<dyn Error + Sync + Send + 'static>> {
        match self {
            Command::ListPeers => {
                info!("Listing all Peers:");
                for peer in behaviour.nodes() {
                    info!("Peer: {}", peer);
                }
                Ok(())
            }
            Command::ListLocalRecipes => {
                match recipes.load_recipes() {
                    Err(e) => {
                        log::error!("Error loading recipes: {}", e);
                        return Ok(());
                    }
                    Ok(recipes) => {
                        info!("Listing my recipes:");
                        for r in recipes {
                            info!("{:?}", r);
                        }
                    }
                }

                Ok(())
            }
            Command::CreateRecipe(name, ingredients, instructions) => {
                match recipes.create_new_recipe(name, ingredients, instructions) {
                    Err(e) => {
                        log::error!("Error creating recipe: {}", e);
                        return Ok(());
                    }
                    Ok(recipe) => info!("Recipe created: {:?}", recipe),
                }
                Ok(())
            }
            Command::ListAllRecipes => {
                info!("Listing all recipes:");

                let message = RecipeMessage::Request(ListRequest {
                    mode: ListMode::ALL,
                });

                match message.serialize_to_bytes() {
                    Ok(data) => behaviour.publish(topic, data),
                    Err(e) => log::error!("Error serializing message: {e}"),
                }

                Ok(())
            }
            Command::ListPeerRecipes(peer_id) => {
                let message = RecipeMessage::Request(ListRequest {
                    mode: ListMode::One(peer_id.clone()),
                });

                match message.serialize_to_bytes() {
                    Ok(data) => behaviour.publish(topic, data),
                    Err(e) => log::error!("Error serializing message: {e}"),
                }

                Ok(())
            }
            Command::Unknown(line) => {
                log::error!("Unknown command: {}", line);
                Ok(())
            }
        }
    }
}

impl From<&str> for Command {
    fn from(s: &str) -> Self {
        match s.split_once(" ") {
            Some(("ls", rest)) => {
                let tokens = rest.split_ascii_whitespace().collect::<Vec<_>>();
                match tokens.as_slice() {
                    ["p"] => Command::ListPeers,
                    ["r"] => Command::ListLocalRecipes,
                    ["r", "all"] => Command::ListAllRecipes,
                    ["r", peer_id] => Command::ListPeerRecipes(peer_id.to_string()),
                    _ => Command::Unknown(s.to_string()),
                }
            }
            Some(("create", recipe)) => {
                let mut parts = recipe.splitn(3, "|");
                let name = parts.next().unwrap_or_default();
                let ingredients = parts.next().unwrap_or_default();
                let instructions = parts.next().unwrap_or_default();
                Command::CreateRecipe(
                    name.to_string(),
                    ingredients.to_string(),
                    instructions.to_string(),
                )
            }
            _ => Command::Unknown(s.to_string()),
        }
    }
}
