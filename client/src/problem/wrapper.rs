use crate::{
    app::API_URL,
    problem::{
    checklist::ChecklistComponent, free_response::FreeRespComponent,
    multiple_choice::MultChoiceComponent,
}};
use common::problems::{Problem, ProblemContent};
use log::*;
use yew::{
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

pub enum ProblemMsg {
    StatusUpdate(ProblemStatus),
    SendRec,
    UndoRec,
    RecSuccess(i32),
    RecFailure(String),
}


#[derive(Debug, Clone, Properties)]
pub struct ProblemProps {
    pub problemid: i32,
}

pub struct ProblemComponent {
    link: ComponentLink<Self>,
    fetch_service: FetchService,
    ft: Option<FetchTask>,
    props: ProblemProps,
    problem: ProblemStatus
}

impl ProblemComponent {
    fn send_request(&mut self) -> FetchTask {
        let callback = self.link.callback(move |response: Response<Json<Result<Problem, anyhow::Error>>>| {
            let (meta, Json(problem_resp)) = response.into_parts();
            ProblemMsg::StatusUpdate(match problem_resp {
                Ok(problem_resp) => if meta.status.is_success() {
                    ProblemStatus::Loaded(problem_resp)
                } else {
                    ProblemStatus::Failed(format!("{}", meta.status))
                },
                Err(error) => ProblemStatus::Failed(format!("ERROR: {}", error))
            })
        });
        let request = Request::get(format!("{}/problems/{}/", API_URL, self.props.problemid))
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
            ft: None,
            problem: ProblemStatus::Loading,
            props,
        };
        component.ft = Some(component.send_request());
        component
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            ProblemMsg::StatusUpdate(status) => {
                self.problem = status;
                true
            },
            _ => unimplemented!("Not all problem messages are handled yet"),
        }
    }

    fn view(&self) -> Html {
        match &self.problem {
            ProblemStatus::Loaded(problem) => html! {
                <div class="problem loaded">
                    <div class="header">
                        <div class="topic">
                            { serde_json::to_string(&problem.topic).unwrap() }
                        </div>
                        <div class="tags">
                            { for problem.tags.iter().map(|tag| html! { <div class="tag">{tag}</div> }) }
                        </div>
                        <div class="prompt">
                            { &problem.prompt }
                        </div>
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
                        <div class="recommend">
                            { problem.recommendations } { "recommendations" }
                        </div>
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
            }
        }
    }
}
