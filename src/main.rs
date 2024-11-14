use crossterm::event::{self, Event, KeyCode};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    widgets::{Block, Borders, Paragraph},
    Terminal,
};
use std::io;
use std::sync::{Arc, Mutex};
use tokio::time::Duration;

// Action: 状態を変更するイベント
enum Action {
    Increment,
    Decrement,
}

// Store: アプリケーションの状態を管理
struct Store {
    count: i32,
}

impl Store {
    fn new() -> Self {
        Self { count: 0 }
    }

    // Actionを受けて状態を更新する
    fn update(&mut self, action: Action) {
        self.count += match action {
            Action::Increment => 1,
            Action::Decrement => -1,
        };
    }
}

// Dispatcher: ActionをStoreに通知
struct Dispatcher {
    store: Arc<Mutex<Store>>,
}

impl Dispatcher {
    fn new(store: Arc<Mutex<Store>>) -> Self {
        Self { store }
    }

    // ActionをDispatcher経由でStoreに送信
    fn dispatch(&self, action: Action) {
        self.store.lock().unwrap().update(action);
    }
}

// View: ユーザーインターフェースを描画し、ユーザー操作に反応
struct View<B: ratatui::backend::Backend> {
    terminal: Terminal<B>,
    dispatcher: Dispatcher,
}

impl<B: ratatui::backend::Backend> View<B> {
    fn new(terminal: Terminal<B>, dispatcher: Dispatcher) -> Self {
        Self {
            terminal,
            dispatcher,
        }
    }

    fn draw_ui(&mut self, count: i32) -> io::Result<()> {
        self.terminal.draw(|f| {
            // レイアウト設定（垂直方向に1つの大きな領域）
            let chunk = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(100)].as_ref())
                .split(f.area())[0]; // 1つの領域を分割

            // 「Counter: x」というテキストを表示するパラグラフ
            f.render_widget(
                Paragraph::new(format!("Counter: {}", count))
                    .block(Block::default().borders(Borders::ALL)),
                chunk,
            );
        })?;
        Ok(())
    }

    async fn run(&mut self) -> io::Result<()> {
        loop {
            let count = self.dispatcher.store.lock().unwrap().count;
            self.draw_ui(count)?;

            if let Some(action) = self.handle_user_input().await {
                self.dispatcher.dispatch(action);
            }

            if event::poll(Duration::from_millis(50))? {
                if let Event::Key(key) = event::read()? {
                    if key.code == KeyCode::Char('q') {
                        break;
                    }
                }
            }
        }
        self.terminal.clear()?;
        Ok(())
    }

    async fn handle_user_input(&self) -> Option<Action> {
        if event::poll(Duration::from_millis(50)).unwrap() {
            if let Event::Key(key) = event::read().unwrap() {
                return match key.code {
                    KeyCode::Up => Some(Action::Increment),
                    KeyCode::Down => Some(Action::Decrement),
                    _ => None,
                };
            }
        }
        None
    }
}

#[tokio::main]
async fn main() -> Result<(), io::Error> {
    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    let store = Arc::new(Mutex::new(Store::new()));
    let dispatcher = Dispatcher::new(store.clone());

    let mut view = View::new(terminal, dispatcher);
    view.run().await?;
    Ok(())
}
