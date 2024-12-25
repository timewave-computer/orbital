```sh
 _______  ______    _______  ___   _______  _______  ___
|       ||    _ |  |  _    ||   | |       ||   _   ||   |
|   _   ||   | ||  | |_|   ||   | |_     _||  |_|  ||   |
|  | |  ||   |_||_ |       ||   |   |   |  |       ||   |
|  |_|  ||    __  ||  _   | |   |   |   |  |       ||   |___
|       ||   |  | || |_|   ||   |   |   |  |   _   ||       |
|_______||___|  |_||_______||___|   |___|  |__| |__||_______|
```

# Orbital

Orbital is a specialized cross-chain intent system designed for use by protocols. Given some `(input_token, input_domain)` tuple, orbital enables transfers and swaps by making declarative statements about the desired `(destination_token, destination_domain)`. Bonded solvers participate in an english auction in which the winner is given exclusive execution rights. The solver's bond is lost if they fail to correctly fill within a timeout.

## Status

This project has been specced and scaffolded. Some iteration of orbital will be used as an overlay for improving the performance of Valence programs.