use std::{
    fs,
    io::{self, BufRead, BufReader, Read, Write as _},
    path::{Path, PathBuf},
    process::{Child, ChildStdin, Command, Stdio},
    sync::mpsc::{self, Receiver},
    thread,
    time::{Duration, Instant},
};

use lsp_types::{
    ClientCapabilities, CompletionClientCapabilities, CompletionContext, CompletionItem,
    CompletionItemCapability, CompletionParams, CompletionResponse, CompletionTriggerKind,
    DidOpenTextDocumentParams, InitializeParams, InitializedParams, PartialResultParams, Position,
    TextDocumentClientCapabilities, TextDocumentIdentifier, TextDocumentItem,
    TextDocumentPositionParams, Uri, WindowClientCapabilities, WorkDoneProgressParams,
    WorkspaceClientCapabilities, WorkspaceFolder,
    notification::{DidOpenTextDocument, Exit, Initialized, Notification},
    request::{Completion, Initialize, Request, Shutdown},
};
use quote::quote;
use serde_json::{Value, json};

#[test]
fn rust_analyzer_completes_validator_dot_chains() {
    if cfg!(coverage) {
        eprintln!("skipping rust-analyzer LSP completion test under coverage");
        return;
    }

    if Command::new("rust-analyzer")
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_err()
    {
        eprintln!("skipping rust-analyzer LSP completion test: rust-analyzer is not on PATH");
        return;
    }

    let workspace_root = workspace_root();
    let fixture_path = workspace_root.join("crates/koruma/tests/integration/fixtures.rs");
    let mut source = fs::read_to_string(&fixture_path).expect("fixture source should be readable");
    source.push_str(&completion_probe_source());

    let string_position = position_after_last(&source, "StringLengthValidation.");
    let generic_position = position_after_last(&source, "NumberRangeValidation::<_>.");
    let uri = file_uri(&fixture_path);

    let mut client = LspClient::spawn(&workspace_root).expect("rust-analyzer should start");
    client.initialize(&workspace_root);
    client.open_document(&uri, &source);

    let string_completions = client.wait_for_completions(
        &uri,
        string_position,
        &[
            ExpectedCompletion {
                label: "min",
                detail_fragment: "usize",
            },
            ExpectedCompletion {
                label: "max",
                detail_fragment: "usize",
            },
        ],
    );
    assert!(
        string_completions,
        "expected StringLengthValidation. to complete typed min/max setters"
    );

    let generic_completions = client.wait_for_completions(
        &uri,
        generic_position,
        &[
            ExpectedCompletion {
                label: "min",
                detail_fragment: "i32",
            },
            ExpectedCompletion {
                label: "max",
                detail_fragment: "i32",
            },
        ],
    );
    assert!(
        generic_completions,
        "expected NumberRangeValidation::<_>. to complete typed min/max setters"
    );

    client.shutdown();
}

struct ExpectedCompletion {
    label: &'static str,
    detail_fragment: &'static str,
}

struct LspClient {
    child: Child,
    stdin: ChildStdin,
    rx: Receiver<Result<Value, String>>,
    next_id: u64,
}

impl LspClient {
    fn spawn(workspace_root: &Path) -> io::Result<Self> {
        let mut child = Command::new("rust-analyzer")
            .arg("--log-file")
            .arg(workspace_root.join("target/koruma-rust-analyzer-lsp-test.log"))
            .current_dir(workspace_root)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()?;
        let stdin = child
            .stdin
            .take()
            .expect("rust-analyzer stdin should be piped");
        let stdout = child
            .stdout
            .take()
            .expect("rust-analyzer stdout should be piped");
        let (tx, rx) = mpsc::channel();

        thread::spawn(move || {
            let mut reader = BufReader::new(stdout);
            loop {
                match read_lsp_message(&mut reader) {
                    Ok(Some(message)) => {
                        if tx.send(Ok(message)).is_err() {
                            break;
                        }
                    },
                    Ok(None) => break,
                    Err(error) => {
                        let _ = tx.send(Err(error.to_string()));
                        break;
                    },
                }
            }
        });

        Ok(Self {
            child,
            stdin,
            rx,
            next_id: 1,
        })
    }

    fn initialize(&mut self, workspace_root: &Path) {
        let root_uri = file_uri(workspace_root);
        let id = self.request::<Initialize>(InitializeParams {
            process_id: Some(std::process::id()),
            initialization_options: Some(rust_analyzer_config()),
            capabilities: ClientCapabilities {
                workspace: Some(WorkspaceClientCapabilities {
                    workspace_folders: Some(true),
                    configuration: Some(true),
                    ..WorkspaceClientCapabilities::default()
                }),
                text_document: Some(TextDocumentClientCapabilities {
                    completion: Some(CompletionClientCapabilities {
                        completion_item: Some(CompletionItemCapability {
                            snippet_support: Some(true),
                            ..CompletionItemCapability::default()
                        }),
                        context_support: Some(true),
                        ..CompletionClientCapabilities::default()
                    }),
                    ..TextDocumentClientCapabilities::default()
                }),
                window: Some(WindowClientCapabilities {
                    work_done_progress: Some(true),
                    ..WindowClientCapabilities::default()
                }),
                ..ClientCapabilities::default()
            },
            workspace_folders: Some(vec![WorkspaceFolder {
                uri: root_uri,
                name: "koruma".to_owned(),
            }]),
            ..InitializeParams::default()
        });
        self.wait_for_response(id, Duration::from_secs(60))
            .expect("rust-analyzer initialize should succeed");
        self.notify::<Initialized>(InitializedParams {});
    }

    fn open_document(&mut self, uri: &Uri, source: &str) {
        self.notify::<DidOpenTextDocument>(DidOpenTextDocumentParams {
            text_document: TextDocumentItem::new(
                uri.clone(),
                "rust".to_owned(),
                1,
                source.to_owned(),
            ),
        });
    }

    fn wait_for_completions(
        &mut self,
        uri: &Uri,
        position: Position,
        expected: &[ExpectedCompletion],
    ) -> bool {
        let deadline = Instant::now() + Duration::from_secs(90);
        while Instant::now() < deadline {
            let items = self.completions(uri, position);
            if expected.iter().all(|expected| {
                items.iter().any(|item| {
                    item.label == expected.label
                        && item
                            .detail
                            .as_deref()
                            .is_some_and(|detail| detail.contains(expected.detail_fragment))
                })
            }) {
                return true;
            }
            thread::sleep(Duration::from_secs(1));
        }
        false
    }

    fn completions(&mut self, uri: &Uri, position: Position) -> Vec<CompletionItem> {
        let id = self.request::<Completion>(CompletionParams {
            text_document_position: TextDocumentPositionParams::new(
                TextDocumentIdentifier::new(uri.clone()),
                position,
            ),
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
            context: Some(CompletionContext {
                trigger_kind: CompletionTriggerKind::INVOKED,
                trigger_character: None,
            }),
        });
        let Some(result) = self
            .wait_for_response(id, Duration::from_secs(30))
            .and_then(|message| message.get("result").cloned())
        else {
            return Vec::new();
        };

        match serde_json::from_value::<Option<CompletionResponse>>(result)
            .expect("completion response should match LSP completion result shape")
        {
            Some(CompletionResponse::Array(items)) => items,
            Some(CompletionResponse::List(list)) => list.items,
            None => Vec::new(),
        }
    }

    fn request<R>(&mut self, params: R::Params) -> u64
    where
        R: Request,
    {
        let id = self.next_id;
        self.next_id += 1;
        self.send(json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": R::METHOD,
            "params": serde_json::to_value(params).expect("LSP request params should serialize"),
        }));
        id
    }

    fn notify<N>(&mut self, params: N::Params)
    where
        N: Notification,
    {
        self.send(json!({
            "jsonrpc": "2.0",
            "method": N::METHOD,
            "params": serde_json::to_value(params).expect("LSP notification params should serialize"),
        }));
    }

    fn wait_for_response(&mut self, id: u64, timeout: Duration) -> Option<Value> {
        let deadline = Instant::now() + timeout;
        while Instant::now() < deadline {
            let remaining = deadline.saturating_duration_since(Instant::now());
            let message = match self
                .rx
                .recv_timeout(remaining.min(Duration::from_millis(250)))
            {
                Ok(Ok(message)) => message,
                Ok(Err(error)) => panic!("rust-analyzer LSP read failed: {error}"),
                Err(_) => continue,
            };

            if message.get("id").and_then(Value::as_u64) == Some(id)
                && (message.get("result").is_some() || message.get("error").is_some())
            {
                if let Some(error) = message.get("error") {
                    panic!("rust-analyzer LSP request {id} failed: {error}");
                }
                return Some(message);
            }

            if message.get("id").is_some() && message.get("method").is_some() {
                self.respond_to_server_request(&message);
            }
        }
        None
    }

    fn respond_to_server_request(&mut self, message: &Value) {
        let id = message
            .get("id")
            .cloned()
            .expect("server request should include id");
        let method = message.get("method").and_then(Value::as_str).unwrap_or("");
        let result = match method {
            "workspace/configuration" => json!([rust_analyzer_config()]),
            _ => Value::Null,
        };
        self.send(json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": result,
        }));
    }

    fn send(&mut self, message: Value) {
        let body = serde_json::to_vec(&message).expect("LSP message should serialize");
        write!(self.stdin, "Content-Length: {}\r\n\r\n", body.len())
            .expect("rust-analyzer stdin should accept header");
        self.stdin
            .write_all(&body)
            .expect("rust-analyzer stdin should accept body");
        self.stdin
            .flush()
            .expect("rust-analyzer stdin should flush");
    }

    fn shutdown(&mut self) {
        let id = self.request::<Shutdown>(());
        let _ = self.wait_for_response(id, Duration::from_secs(5));
        self.notify::<Exit>(());
    }
}

impl Drop for LspClient {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

fn read_lsp_message<R: BufRead + Read>(reader: &mut R) -> io::Result<Option<Value>> {
    let mut content_length = None;
    loop {
        let mut line = String::new();
        if reader.read_line(&mut line)? == 0 {
            return Ok(None);
        }
        if line == "\r\n" {
            break;
        }
        if let Some(value) = line.strip_prefix("Content-Length:") {
            content_length = Some(value.trim().parse::<usize>().map_err(io::Error::other)?);
        }
    }

    let content_length = content_length
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "missing Content-Length"))?;
    let mut body = vec![0; content_length];
    reader.read_exact(&mut body)?;
    serde_json::from_slice(&body)
        .map(Some)
        .map_err(io::Error::other)
}

fn completion_probe_source() -> String {
    let file = syn::parse2::<syn::File>(quote! {
        #[derive(Koruma)]
        pub struct RaCompletionProbeUnique {
            #[koruma(StringLengthValidation.)]
            pub ra_unique_name: String,

            #[koruma(NumberRangeValidation::<_>.)]
            pub ra_unique_age: i32,
        }
    })
    .expect("completion probe source should parse as Rust");

    let mut source = String::from("\n\n");
    source.push_str(&prettyplease::unparse(&file));
    source
}

fn rust_analyzer_config() -> Value {
    json!({
        "cargo": {
            "buildScripts": {
                "enable": true,
            },
        },
        "procMacro": {
            "enable": true,
        },
        "checkOnSave": false,
    })
}

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("crate should live under workspace/crates")
        .to_path_buf()
}

fn position_after_last(source: &str, needle: &str) -> Position {
    let byte_index = source
        .rfind(needle)
        .unwrap_or_else(|| panic!("probe `{needle}` should exist"))
        + needle.len();
    let prefix = &source[..byte_index];
    let line = prefix.bytes().filter(|byte| *byte == b'\n').count();
    let character = prefix
        .rsplit_once('\n')
        .map_or(prefix, |(_, after_newline)| after_newline)
        .chars()
        .count();
    Position::new(line as u32, character as u32)
}

fn file_uri(path: &Path) -> Uri {
    let path = path
        .canonicalize()
        .unwrap_or_else(|_| panic!("path should canonicalize: {}", path.display()));
    let path = file_uri_path(&path.to_string_lossy(), cfg!(windows));
    format!("file://{}", path.replace(' ', "%20"))
        .parse()
        .expect("file URI should parse")
}

fn file_uri_path(path: &str, windows: bool) -> String {
    let mut path = path.replace('\\', "/");
    if windows {
        if let Some(stripped) = path.strip_prefix("//?/") {
            path = stripped.to_owned();
        }
        if !path.starts_with('/') {
            path = format!("/{path}");
        }
    }
    path
}

#[test]
fn file_uri_path_normalizes_windows_drive_paths() {
    assert_eq!(
        file_uri_path(r"D:\a\koruma\koruma\crates\koruma", true),
        "/D:/a/koruma/koruma/crates/koruma"
    );
}

#[test]
fn file_uri_path_normalizes_windows_verbatim_drive_paths() {
    assert_eq!(
        file_uri_path(r"\\?\D:\a\koruma\koruma\crates\koruma", true),
        "/D:/a/koruma/koruma/crates/koruma"
    );
}

#[test]
fn file_uri_path_keeps_unix_paths() {
    assert_eq!(
        file_uri_path("/home/runner/work/koruma/koruma", false),
        "/home/runner/work/koruma/koruma"
    );
}
