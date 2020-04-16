use common::problems;
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

pub enum FreeRespMsg {
    InputChanged(String),
}

#[derive(Debug, Clone, Properties)]
pub struct FreeRespProps {
    pub restrictions: Vec<problems::FreeResponseRestriction>,
    pub solution: Vec<problems::FreeResponseSolution>,
}

pub struct FreeRespComponent {
    link: ComponentLink<Self>,
    props: FreeRespProps,
    error_message: String,
    current_input: String,
}

impl FreeRespComponent {
    pub fn input_changed(&self) -> Callback<InputData> {
        self.link
            .callback(move |input: InputData| FreeRespMsg::InputChanged(input.value))
    }
}

impl Component for FreeRespComponent {
    type Message = FreeRespMsg;
    type Properties = FreeRespProps;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self {
            link,
            props,
            error_message: String::new(),
            current_input: String::new(),
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            FreeRespMsg::InputChanged(input) => {
                self.current_input = input.clone();
                for restriction in &self.props.restrictions {
                    if let Err(error) = restriction.check(input.clone()) {
                        self.error_message = error;
                        return true;
                    }
                }
                self.error_message = String::new();
                true
            }
        }
    }

    fn view(&self) -> Html {
        html! {
            <div class="freeresponse">
                <div class="errorbox">
                    { &self.error_message }
                </div>
                <textarea oninput=self.input_changed()></textarea>
            </div>
        }
    }
}
