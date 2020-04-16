use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Problem {
    pub id: i32,
    pub owner_id: i32,
    pub recommendations: i32,
    pub topic: Topic,
    pub tags: Vec<String>,
    pub prompt: String,
    pub content: ProblemContent,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewProblem {
    pub owner_id: i32,
    pub topic: Topic,
    pub tags: Vec<String>,
    pub prompt: String,
    pub content: ProblemContent,
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
        restrictions: Vec<FreeResponseRestriction>,
        solution: Vec<FreeResponseSolution>,
    },
    MultipleChoice {
        options: Vec<String>,
        solution: usize,
    },
    Checklist {
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
                restrictions: _,
                solution: _,
            } => ProblemType::FreeResponse,
            ProblemContent::MultipleChoice {
                options: _,
                solution: _,
            } => ProblemType::MultipleChoice,
            ProblemContent::Checklist {
                options: _,
                solution: _,
            } => ProblemType::Checklist,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FreeResponseRestriction {
    Imaginary,
    Integer,
    Natural,
    MaxCharacterLength(usize),
    RealInRange {
        start: Option<f64>,
        end: Option<f64>,
    },
    ImaginaryInRange {
        start: Option<f64>,
        end: Option<f64>,
    },
}

impl FreeResponseRestriction {
    /// Checks if a given input passes the restriction.
    /// returns Ok if the response is valid and if it is not
    /// it returns a user-displayable error message.
    pub fn check(&self, response: String) -> Result<(), String> {
        match self {
            FreeResponseRestriction::Imaginary => {
                unimplemented!("Imaginary numbers not yet supported as a solution")
            }
            FreeResponseRestriction::Integer => if response.parse::<i32>().is_ok() {
                Ok(())
            } else {
                Err("Input must a valid integer".to_string())
            },
            FreeResponseRestriction::Natural => if response.parse::<u32>().is_ok() {
                Ok(())
            } else {
                Err("Input must a natural number".to_string())
            },
            FreeResponseRestriction::MaxCharacterLength(length) => if response.len() < *length {
                Ok(())
            } else {
                Err(format!("Input must be {} characters or less", length))
            },
            FreeResponseRestriction::RealInRange { start, end } => match response.parse::<f64>() {
                Ok(num) => {
                    if let Some(start) = start {
                        if num < *start {
                            return Err(format!("Input must be greater than {}", start));
                        }
                    }
                    if let Some(end) = end {
                        if num > *end {
                            return Err(format!("Input must be less than {}", end));
                        }
                    }
                    Ok(())
                }
                Err(_) => Err("Input must be a real number".to_string()),
            },
            FreeResponseRestriction::ImaginaryInRange { start: _, end: _ } => {
                unimplemented!("Imaginary numbers not yet supported as a solution")
            }
        }
    }

}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FreeResponseSolution {
    RealEquals { eq: f64, precision: f64 },
    ImaginaryEquals { eq: f64, precision: f64 },
    TextEquals { eq: String },
}

impl FreeResponseSolution {
    /// Checks if a given input is the solution.
    /// returns true if the response is valid.
    pub fn check(&self, response: String) -> bool {
        match self {
            FreeResponseSolution::RealEquals { eq, precision } => {
                if let Ok(num) = response.parse::<f64>() {
                    (num - eq).abs() < *precision
                } else {
                    false
                }
            }
            FreeResponseSolution::ImaginaryEquals { eq, precision } => {
                unimplemented!("Imaginary numbers not yet supported as a solution")
            }
            FreeResponseSolution::TextEquals { eq } => &response == eq,
        }
    }
}
