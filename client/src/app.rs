use log::*;
use yew::prelude::*;

pub struct App {
    link: ComponentLink<Self>,
}

impl Component for App {
    type Message = ();
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        App { link }
    }

    fn update(&mut self, _: Self::Message) -> ShouldRender {
        true
    }

    fn view(&self) -> Html {
        info!("rendered!");
        html! {
            <div class="app"></div>
        }
    }
}
