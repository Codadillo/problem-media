use crate::{app::API_URL, problem::wrapper::ProblemComponent};
use common::{user::User, problems::{Problem, ProblemContent}};
use log::*;
use yew::{
    format::{Json, Nothing},
    prelude::*,
    services::{
        fetch::{FetchTask, Request, Response},
        FetchService,
    },
};

#[derive(Debug, Clone, Properties)]
pub struct FeedProps {
    pub feed_endpoint: String,
    pub user: User,
}

#[derive(Debug)]
pub enum FeedMsg {
    LoadedProblems(Vec<i32>),
}

pub struct FeedComponent {
    link: ComponentLink<Self>,
    fetch_service: FetchService,
    problems_ft: Option<FetchTask>,
    props: FeedProps,
    problems: Vec<i32>,
}

impl FeedComponent {
    fn send_problems_request(&mut self) -> FetchTask {
        let callback = self.link.callback(move |response: Response<Json<Result<Vec<i32>, anyhow::Error>>>| {
            let (meta, Json(problem_ids)) = response.into_parts();
            match problem_ids {
                Ok(problem_ids) => FeedMsg::LoadedProblems(problem_ids),
                Err(error) => unimplemented!("No error handling if feed request to /problems fails"),
            }
        });
        let request = Request::get(&self.props.feed_endpoint)
            .body(Nothing)
            .unwrap();
        self.fetch_service.fetch(request, callback).unwrap()
    }
}

impl Component for FeedComponent {
    type Message = FeedMsg;
    type Properties = FeedProps;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let mut component = Self {
            link,
            props,
            fetch_service: FetchService::new(),
            problems_ft: None,
            problems: vec![],
        };
        component.problems_ft = Some(component.send_problems_request());
        component
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            FeedMsg::LoadedProblems(problems) => {
                self.problems = problems;
                true
            }
        }
    }

    fn view(&self) -> Html {
        html! {
            <div class="feed">
                {
                    if self.problems.is_empty() { // TODO: loading gif
                        html! {
                            <div class="loading">
                                { "Loading feed" }
                            </div>
                        }
                    } else {
                        html! {
                            <div class="problemswrapper">
                                <div class="problems">
                                    {
                                        for self.problems.iter().map(|problem_id| html! {
                                            <ProblemComponent problemid={ problem_id } recommended=&self.props.user.recommended_ids.contains(&problem_id) />
                                        })
                                    }
                                </div>
                            </div>
                        }
                    }
                }
            </div>
        }
    }
}
