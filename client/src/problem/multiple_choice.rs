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
use yew_router::{route::Route, service::RouteService, Switch};

pub enum MultChoiceMsg {
    ChoiceSelected(usize),
}

#[derive(Debug, Clone, Properties)]
pub struct MultChoiceProps {
    pub options: Vec<String>,
    pub solution: usize,
}

pub struct MultChoiceComponent {
    link: ComponentLink<Self>,
    props: MultChoiceProps,
    choice: Option<usize>,
}

impl MultChoiceComponent {
    pub fn select_choice(&self, idx: usize) -> Callback<MouseEvent> {
        self.link
            .callback(move |_| MultChoiceMsg::ChoiceSelected(idx))
    }
}

impl Component for MultChoiceComponent {
    type Message = MultChoiceMsg;
    type Properties = MultChoiceProps;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self {
            link,
            props,
            choice: None,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            MultChoiceMsg::ChoiceSelected(idx) => {
                self.choice = Some(idx);
            }
        }
        true
    }

    fn view(&self) -> Html {
        html! {
            <div class="multiplechoice">
                {
                    for self.props.options.iter().enumerate().map(|(i, option)| html! {
                        <div onclick=self.select_choice(i) class={
                            if Some(i) == self.choice {
                                "option selected"
                            } else {
                                "option"
                            }
                        }>{option}</div>
                    })
                }
            </div>
        }
    }
}
