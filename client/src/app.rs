use log::*;
use yew::prelude::*;
use yew_router::{route::Route, service::RouteService, Switch};

#[derive(Debug, Switch, Clone)]
pub enum AppRoute {
    #[to = "/login"]
    Login,
}

pub enum Msg {
    RouteChanged(Route<()>),
    ChangeRoute(AppRoute),
}

pub struct App {
    route_service: RouteService<()>,
    route: Route<()>,
    link: ComponentLink<Self>,
}

impl App {
    fn change_route(&self, app_route: AppRoute) -> Callback<Event> {
        self.link.callback(move |_| {
            Msg::ChangeRoute(app_route.clone())
        })
    }
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        let mut route_service: RouteService<()> = RouteService::new();
        let route = route_service.get_route();
        let callback = link.callback(Msg::RouteChanged);
        route_service.register_callback(callback);

        App {
            route_service,
            route,
            link,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::RouteChanged(route) => self.route = route,
            Msg::ChangeRoute(route) => {
                let route_string = match route {
                    AppRoute::Login => "/login".to_string()
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
            <div class="app"></div>
        }
    }
}
