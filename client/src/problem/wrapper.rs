use crate::problem::{
    checklist::ChecklistComponent, free_response::FreeRespComponent,
    multiple_choice::MultChoiceComponent,
};
use common::problems::{Problem, ProblemContent};
use log::*;
use serde::{Deserialize, Serialize};
use yew::{
    format::{Json, Nothing},
    prelude::*,
    services::{
        fetch::{FetchTask, Request, Response},
        FetchService,
    },
};
use yew_router::{route::Route, service::RouteService, Switch};

pub enum ProblemMsg {
    SendRec,
    UndoRec,
    RecSuccess(i32),
    RecFailure(String),
}

#[derive(Debug, Clone, Properties)]
pub struct ProblemProps {
    pub problem: Problem,
}

pub struct ProblemComponent {
    link: ComponentLink<Self>,
    fetch_service: FetchService,
    ft: Option<FetchTask>,
    props: ProblemProps,
}

impl Component for ProblemComponent {
    type Message = ProblemMsg;
    type Properties = ProblemProps;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self {
            link,
            fetch_service: FetchService::new(),
            ft: None,
            props,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        true
    }

    fn view(&self) -> Html {
        html! {
            <div class="problem">
                <div class="header">
                    <div class="topic">
                        { serde_json::to_string(&self.props.problem.topic).unwrap() }
                    </div>
                    <div class="tags">
                        { for self.props.problem.tags.iter().map(|tag| html! { <div class="tag">{tag}</div> }) }
                    </div>
                    <div class="prompt">
                        { &self.props.problem.prompt }
                    </div>
                </div>

                <div class="content">
                    {
                        match &self.props.problem.content {
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
                        { self.props.problem.recommendations } { "recommendations" }
                    </div>
                </div>
            </div>
        }
    }
}
