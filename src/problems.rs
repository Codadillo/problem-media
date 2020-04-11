use crate::database::models::NewDbProblem;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Problem {
    pub id: i32,
    pub owner_id: i32,
    pub topic: Topic,
    pub tags: Vec<String>,
    pub content: ProblemContent,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewProblem {
    pub owner_id: i32,
    pub topic: Topic,
    pub tags: Vec<String>,
    pub content: ProblemContent,
}

impl NewProblem {
    pub fn into_new_db_problem(self) -> Result<NewDbProblem, serde_json::Error> {
        Ok(NewDbProblem {
            owner_id: self.owner_id,
            p_type: serde_json::to_string(&self.content.get_type())?,
            topic: serde_json::to_string(&self.topic)?,
            tags: self.tags,
            data: serde_json::to_string(&self.content)?,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Topic {
    Math,
    Trivia,
    Logic,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProblemContent {
    FreeResponse {
        prompt: String,
        restrictions: Vec<FreeResponseSolution>,
    },
    MultipleChoice {
        prompt: String,
        options: Vec<String>,
        solution: usize,
    },
    Checklist {
        prompt: String,
        options: Vec<String>,
        solution: Vec<usize>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProblemType {
    FreeResponse,
    MultipleChoice,
    Checklist,
}

impl ProblemContent {
    pub fn get_type(&self) -> ProblemType {
        match self {
            ProblemContent::FreeResponse {
                prompt: _,
                restrictions: _,
            } => ProblemType::FreeResponse,
            ProblemContent::MultipleChoice {
                prompt: _,
                options: _,
                solution: _,
            } => ProblemType::MultipleChoice,
            ProblemContent::Checklist {
                prompt: _,
                options: _,
                solution: _,
            } => ProblemType::Checklist,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FreeResponseSolution {
    // These are restrictions users see
    Imaginary,
    Integer,
    Natural,
    MaxCharacterLength(u32),
    RealInRange {
        start: Option<f64>,
        end: Option<f64>,
    },
    ImaginaryInRange {
        start: Option<f64>,
        end: Option<f64>,
    },
    // These determine whether or not a solution is correct
    RealEquals {
        eq: f64,
        precision: f64,
    },
    ImaginaryEquals {
        eq: f64,
        precision: f64,
    },
    TextEquals {
        eq: String,
    },
}
