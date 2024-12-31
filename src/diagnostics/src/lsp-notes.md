{
  "jsonrpc": "2.0",
  "method": "textDocument/publishDiagnostics",
  "params": {
    "uri": "file:///path/to/file",
    "diagnostics": [
      {
        "range": { // Where the issue is located
          "start": { "line": 10, "character": 5 },
          "end": { "line": 10, "character": 15 }
        },
        "severity": 1, // How serious the issue is
        "code": "E001", // A machine-readable identifier for the issue
        "source": "my-compiler", // Who reported the issue
        "message": "Unexpected token: ';'", // Human-readable message
        "relatedInformation": [ // Additional context (optional)
          {
            "location": {
              "uri": "file:///path/to/another/file",
              "range": {
                "start": { "line": 3, "character": 15 },
                "end": { "line": 3, "character": 20 }
              }
            },
            "message": "This token was expected here."
          }
        ]
      }
    ]
  }
}