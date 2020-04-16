use crate::login::LoginComponent;
use log::*;
use yew::prelude::*;
use yew_router::{route::Route, service::RouteService, Switch};

pub const HOST_URL: &'static str = env!("APP_HOST_URL");
pub const API_URL: &'static str = concat!(env!("APP_HOST_URL"), "/api");

#[derive(Debug, Switch, Clone)]
pub enum AppRoute {
    #[to = "/login"]
    Login,
}

pub enum AppMsg {
    RouteChanged(Route<()>),
    ChangeRoute(AppRoute),
}

pub struct App {
    route_service: RouteService<()>,
    route: Route<()>,
    link: ComponentLink<Self>,
}

impl App {
    fn change_route(&self, app_route: AppRoute) -> Callback<MouseEvent> {
        self.link
            .callback(move |_| AppMsg::ChangeRoute(app_route.clone()))
    }
}

impl Component for App {
    type Message = AppMsg;
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        let mut route_service: RouteService<()> = RouteService::new();
        let route = route_service.get_route();
        let callback = link.callback(AppMsg::RouteChanged);
        route_service.register_callback(callback);

        App {
            route_service,
            route,
            link,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            AppMsg::RouteChanged(route) => self.route = route,
            AppMsg::ChangeRoute(route) => {
                let route_string = match route {
                    AppRoute::Login => "/login".to_string(),
                };
                self.route_service.set_route(&route_string, ());
                self.route = Route {
                    route: route_string,
                    state: (),
                };
            }
        }
        true
    }

    fn view(&self) -> Html {
        info!("rendered!");
        html! {
            <div class="app">
                <nav class="menu">
                    <button onclick=&self.change_route(AppRoute::Login)>{"Log In"}</button>
                </nav>
                {
                    match AppRoute::switch(self.route.clone()) {
                        Some(AppRoute::Login) => html! { <LoginComponent></LoginComponent> },
                        None => html! { <div class="notfound">{"404 lol"}</div> },
                    }
                }
            </div>
        }
    }
}
