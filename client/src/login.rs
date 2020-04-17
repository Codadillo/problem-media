use crate::app::{AppRoute, API_URL};
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
use yew_router::{agent::RouteRequest, prelude::*};

#[derive(Debug, Serialize)]
pub struct UserRequest {
    name: String,
    pass: String,
}

#[derive(Debug)]
pub enum LoginMsg {
    ChangeUsername(String),
    ChangePassword(String),
    MakeRequest(bool),
    Failure(String),
    Success,
    NoOp,
}

pub struct LoginComponent {
    link: ComponentLink<Self>,
    fetch_service: FetchService,
    ft: Option<FetchTask>,
    user_request: UserRequest,
    error_message: String,
    router: Box<dyn Bridge<RouteAgent>>,
}

impl LoginComponent {
    fn update_username(&self) -> Callback<InputData> {
        self.link
            .callback(move |event: InputData| LoginMsg::ChangeUsername(event.value))
    }

    fn update_password(&self) -> Callback<InputData> {
        self.link
            .callback(move |event: InputData| LoginMsg::ChangePassword(event.value))
    }

    fn send_request(&mut self, create: bool) -> FetchTask {
        let callback = self.link.callback(move |response: Response<Nothing>| {
            let (meta, _) = response.into_parts();
            if meta.status.is_success() {
                LoginMsg::Success
            } else {
                LoginMsg::Failure(format!("{}", meta.status))
            }
        });
        let path = if create {
            format!("{}/account/create", API_URL)
        } else {
            format!("{}/account/login", API_URL)
        };
        let mut request = Request::post(path)
            .header("Content-Type", "application/json")
            .body(Json(&self.user_request))
            .unwrap();
        self.fetch_service.fetch(request, callback).unwrap()
    }
}

impl Component for LoginComponent {
    type Message = LoginMsg;
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        let callback = link.callback(|_| LoginMsg::NoOp);
        let router = RouteAgent::bridge(callback);

        Self {
            link,
            user_request: UserRequest {
                name: String::new(),
                pass: String::new(),
            },
            fetch_service: FetchService::new(),
            ft: None,
            error_message: String::new(),
            router,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            LoginMsg::ChangeUsername(username) => {
                self.user_request.name = username;
                false
            }
            LoginMsg::ChangePassword(password) => {
                self.user_request.pass = password;
                false
            }
            LoginMsg::MakeRequest(create) => {
                self.ft = Some(self.send_request(create));
                false
            }
            LoginMsg::Failure(error) => {
                self.error_message = error;
                true
            }
            LoginMsg::Success => {
                self.router
                    .send(RouteRequest::ChangeRoute(Route::from("/feed".to_string())));
                false
            }
            LoginMsg::NoOp => false,
        }
    }

    fn view(&self) -> Html {
        html! {
            <div class="loginwrapper">
                <div class="login">
                    <div class="title">
                        { "Sign In" }
                    </div>
                    <input type="text" class="username" oninput=self.update_username() />
                    <br></br>
                    <input type="password" class="password" oninput=self.update_password() />
                    <div class="errorbox">{self.error_message.clone()}</div>
                    <button class="submit" onclick=self.link.callback(move |_| LoginMsg::MakeRequest(false))>{"Login"}</button>
                    <button class="create" onclick=self.link.callback(move |_| LoginMsg::MakeRequest(true))>{"Create"}</button>
                </div>
            </div>
        }
    }
}
