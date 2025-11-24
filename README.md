# Ray tracing in a weekend

<!--toc:start-->

- [Ray tracing in a weekend](#ray-tracing-in-a-weekend)
  - [Additional features](#additional-features)
    - [Cool progress bar](#cool-progress-bar)
    - [Multithreading](#multithreading)
    - [Preview](#preview)
  - [Plans for the future](#plans-for-the-future)
  <!--toc:end-->

I did [Ray tracing in one weekend](https://raytracing.github.io/books/RayTracingInOneWeekend.html) in Rust **with minimal dependencies**. I've made some changes to the original behavior where I felt like it, but the output still results in a ray traced image.
The resulting image will always be exported to a file called "img.ppm", exporting only happens at the _end_ of rendering, all in one\* go.

## Additional features

### Cool progress bar

A small but cool feature is that I made a progress bar that updates in place. It does this by writing directly to the terminal and using ASCII escape codes. It consists of a type of message (currently I only use INFO) a message which represents what the progress bar is about (rendering image, sending jobs, etc.). The progress bar also shows an ETA until it's done. It's calculated based on how long the progress bar has been progressing and how far along it is. This does mean that if it takes gradually more time (for example with a bottom heavy render) the ETA won't really be accurate. Maybe in the future I'll do some more predictions on the ETA, but for now it works fine.

### Multithreading

Additionally, rendering can be done in a multithreaded fashion. The `Camera` struct has a rendering function `threaded_render`. The `threaded_render` function will create a thread pool and queue jobs (through a channel) for every single pixel. Inside these jobs is a channel transmitter that will send back the rendered pixel as well as the location it's supposed to go in the image. When the thread pool no longer has any jobs, aka every pixel has been rendered, the entire image is transformed into a massive string and written all at once. The thread pool implementation isn't perfect, it doesn't handle the case when the job panics and will just crash. I did implement a shared state, which could be easily extended. I assume that's the best way to keep track of paniced threads and reinstate them, but I didn't look it up and I doubt I'll work on it unless if it ever becomes a problem.

### Preview

A winit application is integrated in the codebase. An application can be created by simply passing a width and height into the constructor. It returns an event loop and an application. An event loop has to be passed to the camera. Then when rendering, a preview window will show the render as it's happening! This is honestly quite simple but the addition look amazing!

> [!NOTE]
> Previewing only works on X11 Linux, Wayland is known to not work and Windows is untested.

## Plans if I ever expand this project

- GPU rendering
  - Render pixels with WebGPU which should be **massively** faster.
- PNG
  - Look into PNGs to see if I can directly render to that instead of ppm. ppm is easy, but also takes up quite some space.
- Ray tracing beyond
  - This was pretty fun to do, so maybe somewhere down the line I might do Ray tracing in a week and or Ray tracing the rest of your life.
  - Alternatively, I saw that someone was working on [GPU Tracing](https://github.com/RayTracing/gpu-tracing) maybe I'll do that, who knows
