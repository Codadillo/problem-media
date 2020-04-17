use crate::{
    app::API_URL,
    problem::wrapper::ProblemComponent,
};
use log::*;
use yew::{
    format::{Json, Nothing},
    prelude::*,
    services::{
        fetch::{FetchTask, Request, Response},
        FetchService,
    },
};
use yew_router::{agent::RouteRequest, prelude::*};

pub enum ViewerMsg {
    NoOp,
    ChangeRoute(String),
}

#[derive(Debug, Clone, Properties)]
pub struct ViewerProps {
    pub problemid: i32,
    pub recommended: bool,
}

pub struct SingleViewerComponent {
    link: ComponentLink<Self>,
    router: Box<dyn Bridge<RouteAgent>>,
    props: ViewerProps,
}

impl SingleViewerComponent {
    fn change_route(&self, route: String) -> Callback<MouseEvent> {
        self.link.callback(move |_| ViewerMsg::ChangeRoute(route.clone()))
    }
}

impl Component for SingleViewerComponent {
    type Message = ViewerMsg;
    type Properties = ViewerProps;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let callback = link.callback(|_| ViewerMsg::NoOp);
        let router = RouteAgent::bridge(callback);

        Self {
            link,
            router,
            props
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            ViewerMsg::NoOp => false,
            ViewerMsg::ChangeRoute(route) => {
                self.router
                    .send(RouteRequest::ChangeRoute(Route::from(route)));
                false
            }
        }
    }

    fn view(&self) -> Html {
        html!{
            <div class="singleviewer">
                <ProblemComponent problemid=self.props.problemid recommended=self.props.recommended />
                <button onclick=&self.change_route("/create".to_string())>{ "Create another problem" }</button>
            </div>
        }
    }
}
