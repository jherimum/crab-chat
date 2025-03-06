use std::{error::Error, path::PathBuf};

use serde::{Deserialize, Serialize};
use tokio::{fs::OpenOptions, sync::OnceCell};

static RECIPES: OnceCell<Recipes> = OnceCell::const_new();

pub async fn recipes() -> &'static Recipes {
    RECIPES
        .get_or_init(|| async move { Recipes::new(PathBuf::from("./recipes.json")).await })
        .await
}

pub struct Recipes {
    file: PathBuf,
}

impl Recipes {
    pub async fn new(file: PathBuf) -> Self {
        OpenOptions::new()
            .create(true)
            .write(true)
            .open(file.clone())
            .await
            .unwrap();
        Self { file }
    }

    fn raw_load_recipes(&self) -> Result<String, Box<dyn Error + Sync + Send + 'static>> {
        let file = std::fs::read_to_string(&self.file)?;
        Ok(file)
    }

    pub fn load_recipes(&self) -> Result<Vec<Recipe>, Box<dyn Error + Sync + Send + 'static>> {
        let file = self.raw_load_recipes()?;
        if file.is_empty() {
            return Ok(vec![]);
        }
        let recipes = serde_json::from_str(&file)?;
        Ok(recipes)
    }

    fn write_recipes(
        &self,
        recipes: &[Recipe],
    ) -> Result<(), Box<dyn Error + Sync + Send + 'static>> {
        let file = serde_json::to_string(recipes)?;
        std::fs::write(&self.file, file)?;
        Ok(())
    }

    pub fn create_new_recipe(
        &self,
        name: &str,
        ingredients: &str,
        instructions: &str,
    ) -> Result<Recipe, Box<dyn Error + Sync + Send + 'static>> {
        let recipe = Recipe {
            id: uuid::Uuid::new_v4(),
            name: name.to_string(),
            ingredients: ingredients.to_string(),
            instructions: instructions.to_string(),
            public: true,
        };

        let mut recipes = self.load_recipes()?;
        recipes.push(recipe.clone());

        self.write_recipes(&recipes)?;

        Ok(recipe)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recipe {
    pub id: uuid::Uuid,
    pub name: String,
    pub ingredients: String,
    pub instructions: String,
    pub public: bool,
}
