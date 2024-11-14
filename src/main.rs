use crossterm::event::{self, Event, KeyCode};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    widgets::{Block, Borders, Paragraph},
    Terminal,
};
use std::io;
use std::sync::{Arc, Mutex};
use std::time::Duration;

// Action: 状態を更新するためのイベントを表す
enum Action {
    Increment,
    Decrement,
}

// Store: 状態を管理し、Actionに応じてデータを更新
struct Store {
    count: i32,
}

impl Store {
    fn new() -> Self {
        Self { count: 0 }
    }

    fn update(&mut self, action: Action) {
        match action {
            Action::Increment => self.count += 1,
            Action::Decrement => self.count -= 1,
        }
    }
}

// Dispatcher: ActionをStoreに通知する
struct Dispatcher {
    store: Arc<Mutex<Store>>,
}

impl Dispatcher {
    fn new(store: Arc<Mutex<Store>>) -> Self {
        Self { store }
    }

    fn dispatch(&self, action: Action) {
        let mut store = self.store.lock().unwrap();
        store.update(action);
    }
}

// View: UIを描画し、ユーザー操作に応じてActionを発火
struct View<B: ratatui::backend::Backend> {
    terminal: Terminal<B>,
    dispatcher: Dispatcher,
}

impl<B: ratatui::backend::Backend> View<B> {
    fn new(terminal: Terminal<B>, dispatcher: Dispatcher) -> Self {
        Self { terminal, dispatcher }
    }

    fn draw_ui(&mut self, count: i32) -> io::Result<()> {
        self.terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(100)].as_ref())
                .split(f.area());

            let count_text = format!("Counter: {}", count);
            let paragraph = Paragraph::new(count_text).block(Block::default().borders(Borders::ALL));
            f.render_widget(paragraph, chunks[0]);
        })?;
        Ok(())
    }

    fn handle_user_input(&self) -> Option<Action> {
        if event::poll(Duration::from_millis(50)).unwrap() {
            if let Event::Key(key) = event::read().unwrap() {
                match key.code {
                    KeyCode::Up => Some(Action::Increment),
                    KeyCode::Down => Some(Action::Decrement),
                    KeyCode::Char('q') => return None,
                    _ => None,
                }
            } else {
                None
            }
        } else {
            None
        }
    }

    fn run(&mut self) -> io::Result<()> {
        loop {
            // Storeの状態を取得してUIを描画
            let count = {
                let store = self.dispatcher.store.lock().unwrap();
                store.count
            };
            self.draw_ui(count)?;

            // ユーザー入力の処理
            if let Some(action) = self.handle_user_input() {
                self.dispatcher.dispatch(action);
            }

            // qで終了
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
}

// メイン関数
fn main() -> Result<(), io::Error> {
    // Terminalの初期化
    let stdout = io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    // StoreとDispatcherの作成
    let store = Arc::new(Mutex::new(Store::new()));
    let dispatcher = Dispatcher::new(Arc::clone(&store));

    // Viewの作成と実行
    let mut view = View::new(terminal, dispatcher);
    view.run()?;

    Ok(())
}
