#!/usr/bin/env python3
import json
import subprocess
import sys

def test_command(command_name, params):
    request = {
        "jsonrpc": "2.0",
        "id": 1,
        "method": command_name,
        "params": params
    }
    
    # Run the MCP server
    proc = subprocess.Popen(
        ["cargo", "run", "--quiet"],
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True
    )
    
    # Send request and get response
    stdout, stderr = proc.communicate(json.dumps(request))
    
    try:
        # Extract only the JSON response (last line)
        lines = stdout.strip().split('\n')
        json_response = None
        for line in reversed(lines):
            if line.startswith('{'):
                json_response = line
                break
        
        if json_response:
            response = json.loads(json_response)
            if "result" in response:
                print(f"\n✅ {command_name}:")
                print(json.dumps(response["result"], indent=2))
            else:
                print(f"\n❌ {command_name}:")
                print(json.dumps(response, indent=2))
        else:
            print(f"\n❌ {command_name}: No JSON response found")
    except Exception as e:
        print(f"\n❌ {command_name}: Failed to parse response - {e}")
        print("Response:", stdout)

# Test various commands
test_command("project_structure", {"method": "project_structure"})
test_command("code_metrics", {"method": "code_metrics", "module": "src"})
test_command("expand_snippet", {"method": "expand_snippet", "name": "if_let"})
test_command("analyze_symbol", {"method": "analyze_symbol", "name": "Person"})