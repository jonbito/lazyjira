# VHS Tape: Issue Detail Demo
# Run with: vhs issue-detail.tape

# Configuration
Output issue-detail.gif
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

# Select an issue and open details
Type "j"
Sleep 150ms
Type "j"
Sleep 150ms
Type "j"
Sleep 150ms
Enter
Sleep 1.5s

# Scroll down through the issue details
Type "j"
Sleep 300ms
Type "j"
Sleep 300ms
Type "j"
Sleep 300ms
Type "j"
Sleep 500ms

# Scroll back up
Type "k"
Sleep 300ms
Type "k"
Sleep 500ms

# Jump to top
Type "g"
Sleep 500ms

# Jump to bottom
Type "G"
Sleep 500ms

# Show status change modal (then cancel)
Type "s"
Sleep 1.5s
Escape
Sleep 500ms

# Show add comment modal (then cancel)
Type "c"
Sleep 1.5s
Escape
Sleep 500ms

# Go back to list
Type "q"
Sleep 500ms

# Quit application
Type "q"
Sleep 500ms
