use crate::app::{API_URL, AppRoute};
use log::*;
use yew::{
    format::{Json, Nothing},
    prelude::*,
    services::{FetchService, fetch::{FetchTask, Request, Response},},
};
use yew_router::{route::Route, service::RouteService, Switch};
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize)]
pub struct UserRequest {
    name: String,
    pass: String,
}

#[derive(Debug)]
pub enum LoginMsg {
    ChangeUsername(String),
    ChangePassword(String),
    MakeRequest,
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

    fn send_request(&mut self) -> FetchTask {
        let callback = self.link.callback(
            move |response: Response<Nothing>| {
                let (meta, _) = response.into_parts();
                if meta.status.is_success() {
                    LoginMsg::Success
                } else {
                    LoginMsg::Failure(format!("{}", meta.status))
                }
            },
        );
        let raw_request = serde_json::to_string(&self.user_request).unwrap();
        let request = Request::post(format!("{}/account/login", API_URL))
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
        Self {
            link,
            user_request: UserRequest {
                name: String::new(),
                pass: String::new(),
            },
            fetch_service: FetchService::new(),
            ft: None,
            error_message: String::new(),
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
            LoginMsg::MakeRequest => {
                self.ft = Some(self.send_request());
                false
            }
            LoginMsg::Failure(error) => {
                self.error_message = error;
                true
            }
            LoginMsg::NoOp => false,
            _ => unimplemented!("SUCCESS NOT HANDLED IN UPDATE"),
        }
    }

    fn view(&self) -> Html {
        html! {
            <div class="container">
                <div class="errorbox">{self.error_message.clone()}</div>
                <input type="text" class="username" oninput=self.update_username() />
                <br></br>
                <input type="password" class="password" oninput=self.update_password() />
                <br></br>
                <button class="submit" onclick=self.link.callback(move |_| LoginMsg::MakeRequest)>{"Login"}</button>
            </div>
        }
    }
}
