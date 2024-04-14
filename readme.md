
# Instructions

Go to [the Cargo and Rust installer website](https://doc.rust-lang.org/cargo/getting-started/installation.html)

Download the installer for your device and run it, or enter the installer script into your command line for macOS and Linux.

Follow through all of the prompts, selecting defaults for everything.

Note you may have to install CMake if you don't have it already. 

Once this is all done, clone this repository to your device.

Enter the newly created folder, and run `cargo run`. This will install all of the dependencies for the project and run it. 

To experiment with the project, use the UI menu on the left to change the simulation parameters, and the descriptional UI on the right to observe their effects. To start, I'd recommend pressing the "Reset" button a few times until you get a small web, around 300 strands. Larger webs perform more poorly. You can then press the "Simulation Running" checkbox to start the simulation, and add bugs to see how they collide with the web.

You can change the simulation's parameters, but note that changing the web generation parameters out of balance may cause the simulation to behave unexpectedly. 

In the simulation, you can press `-` and `+` to zoom in and out, and press the left arrow key and right arrow key to rotate the view.

![image](https://github.com/christianbookout/spiderweb-sim/assets/23156778/58393e4f-6ef0-4a4e-9332-6972ae79077b)
