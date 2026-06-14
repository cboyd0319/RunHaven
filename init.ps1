$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest

Write-Host '== Harness verification for RunHaven =='
Write-Host 'Detected stack: python'

$PythonBin = if ($env:PYTHON) {
  $env:PYTHON
} elseif (Get-Command python -ErrorAction SilentlyContinue) {
  'python'
} elseif (Get-Command python3 -ErrorAction SilentlyContinue) {
  'python3'
} else {
  'python'
}

Write-Host '== python3 -m compileall src tests scripts =='
& $PythonBin -m compileall src tests scripts

Write-Host '== PYTHONPATH=src python3 -m unittest discover -s tests =='
$PreviousPythonPath = $env:PYTHONPATH
try {
  $env:PYTHONPATH = 'src'
  & $PythonBin -m unittest discover -s tests
} finally {
  $env:PYTHONPATH = $PreviousPythonPath
}

Write-Host '== python3 scripts/check_pins.py =='
& $PythonBin scripts/check_pins.py

Write-Host '== python3 -m ruff check . =='
& $PythonBin -m ruff check .

Write-Host '== python3 -m mypy src =='
& $PythonBin -m mypy src

Write-Host '== python3 -m build =='
& $PythonBin -m build

Write-Host '== Harness verification complete =='
