use crate::{problem::feed::FeedComponent, login::LoginComponent};
use log::*;
use yew::prelude::*;
use yew_router::{prelude::*, Switch};

pub const HOST_URL: &'static str = "http://localhost:8000"/*env!("APP_HOST_URL")*/;
pub const API_URL: &'static str = concat!("http://localhost:8080"/*env!("APP_HOST_URL")*/, "/api");

#[derive(Debug, Switch, Clone)]
pub enum AppRoute {
    #[to = "/feed"]
    Feed,
    #[to = "/login"]
    Login,
}

pub struct App {
    link: ComponentLink<Self>,
}

impl Component for App {
    type Message = ();
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        App {
            link,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        true
    }

    fn view(&self) -> Html {
        info!("rendered!");
        html! {
            <div class="app">
                <nav class="menu">
                <RouterButton<AppRoute> route=AppRoute::Login>{"Log In"}</RouterButton<AppRoute>>
                </nav>
                <Router<AppRoute>
                    render = Router::render(|switch: AppRoute| {
                        match switch {
                            AppRoute::Login => html! { <LoginComponent></LoginComponent> },
                            AppRoute::Feed => html! { <FeedComponent feed_endpoint=format!("{}/problems/?", API_URL)></FeedComponent> },
                        }
                    })
                />
            </div>
        }
    }
}
