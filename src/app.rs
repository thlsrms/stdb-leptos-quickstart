#[cfg(feature = "hydrate")]
use crate::stdb_chat::*;

#[cfg(feature = "hydrate")]
use std::rc::Rc;

#[cfg(feature = "hydrate")]
use chrono::DateTime;
use leptos::{
    ev::MouseEvent,
    html::{Div, Input},
    prelude::*,
};
use leptos_meta::{provide_meta_context, MetaTags, Stylesheet, Title};
use leptos_router::{
    components::{Route, Router, Routes},
    StaticSegment,
};
#[cfg(feature = "hydrate")]
use spacetimedb_sdk::credentials::cookies::Cookie;
#[cfg(feature = "hydrate")]
use spacetimedb_sdk::{DbConnectionBuilder, DbContext, Identity, Table};

pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
      <!DOCTYPE html>
      <html lang="en">
        <head>
          <meta charset="utf-8" />
          <meta name="viewport" content="width=device-width, initial-scale=1" />
          <AutoReload options=options.clone() />
          <HydrationScripts options />
          <MetaTags />
        </head>
        <body>
          <App />
        </body>
      </html>
    }
}

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    view! {
      <Stylesheet id="leptos" href="/pkg/quickstart-chat.css" />
      <Title text="SpacetimeDB's QuickStart Chat with Leptos" />

      <Router>
        <main>
          <Routes fallback=|| "Page not found.".into_view()>
            <Route path=StaticSegment("") view=HomePage />
          </Routes>
        </main>
      </Router>
    }
}

#[cfg(feature = "hydrate")]
const MODULE_NAME: &str = "quickstart-chat";
#[cfg(feature = "hydrate")]
const HOST: &str = "http://localhost:3000";

#[cfg(feature = "hydrate")]
fn stdb_connection_builder() -> DbConnectionBuilder<RemoteModule> {
    let builder = DbConnection::builder()
        .with_module_name(MODULE_NAME)
        .with_uri(HOST)
        .with_light_mode(true);

    if let Ok(token) = Cookie::get("quickstart-chat_token") {
        builder.with_token(token)
    } else {
        builder
    }
}

#[cfg(feature = "hydrate")]
fn message_on_insert_handler(
    ctx: &EventContext,
    msg: &Message,
    messages_writer: WriteSignal<Vec<(Message, String)>>,
) {
    let sender = ctx
        .db()
        .user()
        .identity()
        .find(&msg.sender)
        .map(|u| u.name.unwrap_or_else(|| "unknown".to_string()))
        .unwrap();

    messages_writer.update(|msgs| {
        msgs.sort_by_key(|(m, _)| m.sent);
        msgs.push((msg.clone(), sender))
    });
}

#[cfg(feature = "hydrate")]
fn register_callbacks(ctx: &DbConnection, messages_writer: WriteSignal<Vec<(Message, String)>>) {
    ctx.subscription_builder()
        .subscribe(["SELECT * FROM user", "SELECT * FROM message"]);

    ctx.db().message().on_insert(move |ctx, msg| {
        message_on_insert_handler(ctx, msg, messages_writer);
    });
}

#[component]
fn HomePage() -> impl IntoView {
    #[cfg(feature = "hydrate")]
    let (identity, set_identity) = signal::<Option<(Identity, String)>>(None);
    #[cfg(feature = "hydrate")]
    let connected = move || identity.get().is_some();
    #[cfg(not(feature = "hydrate"))]
    let connected = move || false;

    let name = RwSignal::<Option<String>>::new(None);
    #[cfg(feature = "hydrate")]
    let (messages, set_messages) = signal::<Vec<(Message, String)>>(Vec::new());

    #[cfg(feature = "hydrate")]
    let connection: LocalResource<Option<Rc<DbConnection>>> =
        LocalResource::new(move || async move {
            if name.get().is_some() && !name.get().unwrap().is_empty() {
                let conn_builder = stdb_connection_builder()
                    .on_connect(move |ctx, id, token| {
                        register_callbacks(ctx, set_messages.clone());
                        let _ = Cookie::new("quickstart-chat_token", token).set();
                        set_identity.set(Some((id, token.into())));
                        let _ = ctx.reducers.set_name(name.get_untracked().unwrap().clone());
                    })
                    .on_disconnect(move |_, error| {
                        set_identity.set(None);
                        set_messages.set(Vec::new());
                        if let Some(err) = error {
                            leptos::logging::error!("Disconnected with error: {:?}", err);
                        }
                    });

                match conn_builder.build().await {
                    Ok(conn) => {
                        conn.run_threaded();
                        return Some(Rc::new(conn));
                    }
                    Err(e) => {
                        leptos::logging::error!("Connection failed with error: {:?}", e);
                        return None;
                    }
                }
            }
            None
        });

    #[cfg(feature = "hydrate")]
    Effect::new(move |_| {
        if connected() {
            let name = name.get().unwrap();
            leptos::logging::log!("{name} connected with id {}", identity.get().unwrap().0);
        }
    });

    let name_input: NodeRef<Input> = NodeRef::new();
    let message_input: NodeRef<Input> = NodeRef::new();
    let message_list_ref: NodeRef<Div> = NodeRef::new();

    #[cfg(feature = "hydrate")]
    Effect::new(move |_| {
        messages.get();
        if let Some(element) = message_list_ref.get() {
            // Scroll down on new messages
            let _ = element.set_scroll_top(element.scroll_height());
        }
    });

    let connect_button_handler = move |e: MouseEvent| {
        e.prevent_default();
        if let Some(input) = name_input.get() {
            if !input.value().is_empty() {
                name.set(Some(input.value()));
            }
        }
    };

    let send_message_handler = move |e: MouseEvent| {
        e.prevent_default();
        #[cfg(feature = "hydrate")]
        if let Some(input) = message_input.get() {
            let content = input.value();
            if !content.is_empty() {
                if let Some(conn) = connection.get().as_deref().flatten() {
                    if let Some(new_name) = content.strip_prefix("/name ") {
                        let _ = conn.reducers.set_name(new_name.into()).unwrap();
                    } else {
                        let _ = conn.reducers.send_message(content).unwrap();
                    }
                }
            }
            input.set_value("");
        }
    };

    view! {
      <div class="chat-container">
        <Show
          when=move || name.get().is_some()
          fallback=move || {
            view! {
              <div class="connect-box">
                <input
                  type="text"
                  placeholder="Enter your name"
                  node_ref=name_input
                  class="name-input"
                />
                <button type="button" on:click=connect_button_handler class="connect-button">
                  "Connect to chat"
                </button>
              </div>
            }
          }
        >
          <Suspense fallback=move || view! { <p>"Connecting..."</p> }>
            <div class="chat-area">
              <Show
                when=move || connected()
                fallback=move || {
                  view! {
                    <div>
                      <p>"Not Connected"</p>
                      <button
                        type="button"
                        class="connect-button"
                        on:click=move |_| {
                          #[cfg(feature = "hydrate")] connection.refetch();
                        }
                      >
                        "Try connecting"
                      </button>

                    </div>
                  }
                }
              >
                <div class="message-list-container" node_ref=message_list_ref>
                  <ul class="message-list">
                    {#[cfg(feature = "hydrate")]
                    {
                      move || {
                        messages
                          .get()
                          .iter()
                          .enumerate()
                          .map(|(index, (msg, sender))| {
                            let datetime = DateTime::parse_from_rfc3339(
                                &msg.sent.to_rfc3339().unwrap(),
                              )
                              .unwrap();
                            let formatted_time = datetime.format("%Y-%m-%d %H:%M").to_string();
                            let row_class = if index % 2 == 0 { "even-row" } else { "odd-row" };

                            view! {
                              <li class=row_class>
                                <span class="message-time">{format!("{}", formatted_time)}</span>
                                <span class="message-content">{format!("{}", msg.text)}</span>
                                <span class="message-sender">{format!("< {}", sender)}</span>
                              </li>
                            }
                          })
                          .collect_view()
                      }
                    }}
                  </ul>
                </div>
                <form class="send-form">
                  <input
                    type="text"
                    placeholder="Type a message"
                    node_ref=message_input
                    class="message-input"
                  />
                  <button type="submit" class="send-button" on:click=send_message_handler>
                    "Send"
                  </button>
                </form>
              </Show>
            </div>
          </Suspense>
        </Show>
      </div>
    }
}
