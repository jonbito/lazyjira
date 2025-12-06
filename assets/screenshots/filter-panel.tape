# VHS Tape: Filter Panel Demo
# Run with: vhs filter-panel.tape

# Configuration
Output filter-panel.gif
Set FontSize 14
Set Width 1200
Set Height 800
Set Theme "Catppuccin Mocha"
Set Padding 20
Set Framerate 30

# Terminal setup
Set TypingSpeed 50ms

# Start the application
Type "lazyjira"
Enter
Sleep 2s

# Open filter panel
Type "f"
Sleep 1s

# Navigate through filter sections
Tab
Sleep 500ms
Tab
Sleep 500ms
Tab
Sleep 500ms

# Go back to first section
Tab
Sleep 500ms

# Navigate filter options
Down
Sleep 300ms
Down
Sleep 300ms

# Toggle a selection
Space
Sleep 500ms

# Move to another section
Tab
Sleep 500ms

# Select another filter
Down
Sleep 300ms
Space
Sleep 500ms

# Cancel without applying
Escape
Sleep 500ms

# Show JQL input mode
Type ":"
Sleep 500ms
Type "project = DEMO AND status = 'In Progress'"
Sleep 1.5s
Escape
Sleep 500ms

# Quit
Type "q"
Sleep 500ms
