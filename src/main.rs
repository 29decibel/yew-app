use yew::prelude::*;

enum Msg {
    AddOne,
}

struct Model {
    value: i64,
}

impl Component for Model {
    type Message = Msg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self { value: 0 }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::AddOne => {
                self.value += 1;
                // the value has changed so we need to
                // re-render for it to appear on the page
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        // This gives us a component's "`Scope`" which allows us to send messages, etc to the component.
        let link = ctx.link();
        html! {
            <div>
                <button onclick={link.callback(|_| Msg::AddOne)}>{ "+1" }</button>
                <p>{ self.value }</p>
            </div>
        }
    }
}

use glob::glob_with;
use glob::MatchOptions;

#[derive(Debug)]
struct Post {
    relative_path: String,
    tags: Vec<String>,
    title: String,
    description: String,
    draft: bool,
    content: String,
}

fn get_posts(dir: &str) -> std::vec::Vec<std::path::PathBuf> {
    let options = MatchOptions {
        case_sensitive: false,
        require_literal_separator: false,
        require_literal_leading_dot: false,
    };
    let markdown_glob = "content/posts/*.md";
    glob_with(format!("{}/{}", dir, markdown_glob).as_str(), options)
        .unwrap()
        .filter_map(|p| p.ok())
        .collect::<Vec<_>>()
}

fn get_post_from_path(path: &std::path::Path) -> Post {
    // reading the path and parse
    Post {
        relative_path: String::from(path.to_str().unwrap()),
        tags: vec![String::from("tag1"), String::from("tag2")],
        title: String::from("some title"),
        description: String::from("some description"),
        draft: false,
        content: String::from(""),
    }
}

fn convert_to_posts(dir: &str, post_paths: std::vec::Vec<std::path::PathBuf>) -> Vec<Post> {
    let nice: Vec<Post> = post_paths
        .iter()
        .filter_map(|pb| pb.strip_prefix(dir).ok())
        .map(|p| get_post_from_path(p))
        .collect();

    nice
}

fn main() {
    let dir = "/Users/dongbinli/sites/orchardlabdev-site";
    let results = get_posts(dir);
    let posts = convert_to_posts(dir, results);
    // parse all the results and convert them into Post array
    println!("{:?}", posts);
    //yew::start_app::<Model>();
}
