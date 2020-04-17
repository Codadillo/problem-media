use crate::{
    app::API_URL,
    problem::variants::{
        checklist::ChecklistComponent, free_response::FreeRespComponent,
        multiple_choice::MultChoiceComponent,
    },
};
use common::problems::{Problem, ProblemContent};
use log::*;
use yew::{
    virtual_dom::{VNode, VText},
    format::{Json, Nothing},
    prelude::*,
    services::{
        fetch::{FetchTask, Request, Response},
        FetchService,
    },
};

pub enum ProblemStatus {
    Loading,
    Loaded(Problem),
    Failed(String),
}

pub enum PromptPart {
    Text(String),
    Latex(String),
}

pub enum ProblemMsg {
    StatusUpdate(ProblemStatus),
    ToggleRec,
    RecSuccess(i32),
    RecFailure(String),
}

#[derive(Debug, Clone, Properties)]
pub struct ProblemProps {
    pub problemid: i32,
    pub recommended: bool,
}

pub struct ProblemComponent {
    link: ComponentLink<Self>,
    fetch_service: FetchService,
    problem_ft: Option<FetchTask>,
    rec_ft: Option<FetchTask>,
    props: ProblemProps,
    problem: ProblemStatus,
    problem_prompt: Vec<PromptPart>,
}

impl ProblemComponent {
    fn toggle_recommend(&self) -> Callback<MouseEvent> {
        self.link.callback(move |_| ProblemMsg::ToggleRec)
    }

    fn send_problem_request(&mut self) -> FetchTask {
        let callback = self.link.callback(
            move |response: Response<Json<Result<Problem, anyhow::Error>>>| {
                let (meta, Json(problem_resp)) = response.into_parts();
                ProblemMsg::StatusUpdate(match problem_resp {
                    Ok(problem_resp) => {
                        if meta.status.is_success() {
                            ProblemStatus::Loaded(problem_resp)
                        } else {
                            ProblemStatus::Failed(format!("{}", meta.status))
                        }
                    }
                    Err(error) => ProblemStatus::Failed(format!("ERROR: {}", error)),
                })
            },
        );
        let request = Request::get(format!("{}/problems/{}/", API_URL, self.props.problemid))
            .body(Nothing)
            .unwrap();
        self.fetch_service.fetch(request, callback).unwrap()
    }

    fn send_rec_request(&mut self) -> FetchTask {
        let callback = self.link.callback(
            move |response: Response<Json<Result<i32, anyhow::Error>>>| {
                let (meta, Json(problem_resp)) = response.into_parts();
                match problem_resp {
                    Ok(problem_resp) => {
                        if meta.status.is_success() {
                            ProblemMsg::RecSuccess(problem_resp)
                        } else {
                            ProblemMsg::RecFailure(format!("{}", meta.status))
                        }
                    }
                    Err(error) => ProblemMsg::RecFailure(format!("ERROR: {}", error)),
                }
            },
        );
        let request = Request::get(format!(
            "{}/problems/{}/recommend/{}",
            API_URL, self.props.problemid, self.props.recommended
        ))
        .body(Nothing)
        .unwrap();
        self.fetch_service.fetch(request, callback).unwrap()
    }
}

impl Component for ProblemComponent {
    type Message = ProblemMsg;
    type Properties = ProblemProps;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let mut component = Self {
            link,
            fetch_service: FetchService::new(),
            problem_ft: None,
            rec_ft: None,
            problem: ProblemStatus::Loading,
            problem_prompt: vec![],
            props,
        };
        component.problem_ft = Some(component.send_problem_request());
        component
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            ProblemMsg::StatusUpdate(status) => {
                match &status {
                    ProblemStatus::Loaded(problem) => {
                        self.problem_prompt = vec![];
                        for (i, part) in problem.prompt.split("$$").enumerate() {
                            self.problem_prompt.push(
                                if i % 2 == 0 {
                                    PromptPart::Text(part.to_string())
                                } else {
                                    PromptPart::Latex(part.to_string())
                                }
                            );
                        }
                    },
                    _ => (),
                }
                self.problem = status;
                true
            }
            ProblemMsg::ToggleRec => {
                if let ProblemStatus::Loaded(problem) = &self.problem {
                    self.rec_ft = Some(self.send_rec_request());
                } else {
                    info!("Attempt to toggle recommendation before problem load");
                }
                true
            }
            ProblemMsg::RecSuccess(rec_count) => {
                if let ProblemStatus::Loaded(problem) = &mut self.problem {
                    problem.recommendations = rec_count;
                    self.props.recommended = !self.props.recommended;
                } else {
                    info!("Recieved rec success before problem load");
                }
                true
            }
            ProblemMsg::RecFailure(error_message) => {
                info!("Error when making rec: {}", error_message);
                false
            }
        }
    }

    fn view(&self) -> Html {
        match &self.problem {
            ProblemStatus::Loaded(problem) => html! {
                <div class="problem loaded">
                    <div class="headerwrapper">
                        <div class="header">
                            <div class="topic">
                                { serde_json::to_string(&problem.topic).unwrap().replace(r#"""#, "").replace("Trivia", "Linguistics") }
                            </div>
                            <div class="tags">
                                { for problem.tags.iter().map(|tag| html! { <div class="tag">{tag}</div> }) }
                            </div>
                        </div>
                    </div>

                    <div class="prompt">
                            {
                                for self.problem_prompt.iter().map(|part| match part {
                                    PromptPart::Text(text) => html! {
                                        <span class="part">{ text }</span>
                                    },
                                    PromptPart::Latex(text) => html! {
                                        <span class="part rendermath">{ text }</span>
                                    }
                                })
                            }
                        </div>

                    <div class="content">
                        {
                            match &problem.content {
                                ProblemContent::FreeResponse { restrictions, solution } => {
                                    html! {
                                        <FreeRespComponent restrictions={ restrictions } solution={ solution } />
                                    }
                                }
                                ProblemContent::MultipleChoice { options, solution } => {
                                    html! {
                                        <MultChoiceComponent options={ options } solution={ solution } />
                                    }
                                }
                                ProblemContent::Checklist { options, solution } => {
                                    html! {
                                        <ChecklistComponent options={ options } solution={ solution } />
                                    }
                                }
                            }
                        }
                    </div>

                    <div class="footer">
                        <div class="reccount">
                            { problem.recommendations } { " recs" }
                        </div>
                        {
                            if self.props.recommended {
                                html! {
                                    <div onclick=&self.toggle_recommend() class="rec true">
                                        { "Unrecommend" }
                                    </div>
                                }
                            } else {
                                html! {
                                    <div onclick=&self.toggle_recommend() class="rec false">
                                        { "Recommend!" }
                                    </div>
                                }
                            }
                        }
                    </div>
                </div>
            },
            ProblemStatus::Loading => html! {
                <div class="problem loading"> // TODO: loading gif
                    { "Loading problem" }
                </div>
            },
            ProblemStatus::Failed(error) => html! {
                <div class="problem failed">
                    { error }
                </div>
            },
        }
    }
}
