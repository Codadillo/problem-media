use crate::{
    login::LoginComponent,
    problem::{create::CreateComponent, feed::FeedComponent, single_viewer::SingleViewerComponent},
};
use common::user::User;
use log::*;
use yew::{
    format::{Json, Nothing},
    prelude::*,
    services::{
        fetch::{FetchTask, Request, Response},
        FetchService,
    },
};
use yew_router::{agent::RouteRequest, prelude::*, Switch};

pub const HOST_URL: &'static str = "http://localhost:8000"/*env!("APP_HOST_URL")*/;
pub const API_URL: &'static str = concat!(
    "http://localhost:8081", /*env!("APP_HOST_URL")*/
    "/api"
);

#[derive(Debug, Switch, Clone)]
pub enum AppRoute {
    #[to = "/create"]
    Create,
    #[to = "/feed"]
    Feed,
    #[to = "/login"]
    Login,
}

pub enum AppMsg {
    RouteUpdate(Route<()>),
    UserLoaded(User),
    UserLoadFail(String),
}

pub struct App {
    link: ComponentLink<Self>,
    fetch_service: FetchService,
    ft: Option<FetchTask>,
    user: Option<User>,
    router: Box<dyn Bridge<RouteAgent>>,
    route: Option<Route<()>>,
}

impl App {
    fn send_user_request(&mut self) -> FetchTask {
        let callback = self.link.callback(
            move |response: Response<Json<Result<User, anyhow::Error>>>| {
                let (meta, Json(user)) = response.into_parts();
                if meta.status.is_success() {
                    match user {
                        Ok(user) => AppMsg::UserLoaded(user),
                        Err(error) => AppMsg::UserLoadFail(format!("{}", error)),
                    }
                } else {
                    AppMsg::UserLoadFail(format!("{}", meta.status))
                }
            },
        );
        let request = Request::get(format!("{}/account/", API_URL))
            .body(Nothing)
            .unwrap();
        self.fetch_service.fetch(request, callback).unwrap()
    }
}

impl Component for App {
    type Message = AppMsg;
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        let callback = link.callback(|route| AppMsg::RouteUpdate(route));
        let router = RouteAgent::bridge(callback);

        let mut app = App {
            link,
            router,
            fetch_service: FetchService::new(),
            ft: None,
            user: None,
            route: None,
        };
        app.router.send(RouteRequest::GetCurrentRoute);
        app
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            AppMsg::RouteUpdate(route) => {
                if route != Route::from("/login".to_string()) {
                    self.ft = Some(self.send_user_request());
                }
                self.route = Some(route);
                true
            }
            AppMsg::UserLoaded(user) => {
                self.user = Some(user);
                true
            }
            AppMsg::UserLoadFail(_error) => {
                self.router
                    .send(RouteRequest::ChangeRoute(Route::from("/login".to_string())));
                true
            }
        }
    }

    fn view(&self) -> Html {
        info!("rendered!");
        info!("{:?}", self.route);
        if let Some(user) = &self.user {
            let user = user.clone();
            html! {
                <div class="app">
                    <nav class="menu">
                        <span class="title">
                            { "AKSHAR" }
                        </span>
                        <div class="routes">
                            <div class="createroute">
                                <RouterButton<AppRoute> route=AppRoute::Create>{"Create Problems"}</RouterButton<AppRoute>>
                            </div>
                            <div class="feedroute">
                                <RouterButton<AppRoute> route=AppRoute::Feed>{"Main Feed"}</RouterButton<AppRoute>>
                            </div>
                            <div class="loginroute">
                                <RouterButton<AppRoute> route=AppRoute::Login>{"Sign Out"}</RouterButton<AppRoute>>
                            </div>
                        </div>
                    </nav>
                    <Router<AppRoute>
                        render = Router::render(move |switch: AppRoute| {
                            match switch {
                                AppRoute::Login => html! { <LoginComponent></LoginComponent> },
                                AppRoute::Feed => html! { <FeedComponent user=user.clone() feed_endpoint=format!("{}/problems/?", API_URL) /> },
                                AppRoute::Create => html! { <CreateComponent user_id=user.id /> },
                            }
                        })
                    />
                </div>
            }
        } else if self.route == Some(Route::from("/login".to_string())) {
            html! {
                <LoginComponent />
            }
        } else {
            html! {
                <div class="app">
                    <div class="apploading">
                        { "Loading app" } // TODO: real loading screen
                    </div>
                </div>
            }
        }
    }
}
