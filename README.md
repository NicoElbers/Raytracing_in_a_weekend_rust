# Ray tracing in a weekend

<!--toc:start-->
- [Ray tracing in a weekend](#ray-tracing-in-a-weekend)
  - [Additional features](#additional-features)
    - [Cool progress bar](#cool-progress-bar)
    - [Multithreading](#multithreading)
  - [Plans for the future](#plans-for-the-future)
<!--toc:end-->

I did [Ray tracing in one weekend](https://raytracing.github.io/books/RayTracingInOneWeekend.html) in Rust without any external dependencies. I've made some changes to the original behavior where I felt like it, but the output still results in a ray traced image. 
The resulting image will always be exported to a file called "img.ppm", exporting only happens at the *end* of rendering, all in one* go.

## Additional features
### Cool progress bar
A small but cool feature is that I made a progress bar that updates in place. It does this by writing directly to the terminal and using ASCII escape codes. It's not much, but it's more pleasing to watch than a constant newline.

### Multithreading
Additionally, rendering can be done in a multithreaded fashion. The `Camera` struct has 2 rendering functions, `render` and `threaded_render`. The `threaded_render` function will create a thread pool and queue jobs for every single pixel. Inside these jobs is a channel transmitter that will send back the rendered pixel as well as the location it's supposed to go in the image. When the thread pool no longer has any jobs, aka every pixel has been rendered, the entire image is transformed into a massive string and written all at once.

## Plans for the future
I plan to still add/ improve the following:
- The progress behavior
    - It currently doesn't work for single threaded rendering, and I can make it cooler with an ETA for when the render is finished.
- Preview application
    - I plan to create a winnit application that will show the image as pixels are coming in. This'll probably tank performance, but idc, it's cool.
- GPU rendering
    - Eventually I plan to render my pixels with WebGPU which should be **massively** faster. But I've never worked with the gpu so we'll see.
- PNG
    - I'm planning to look into PNGs to see if I can directly render to that instead of ppm. ppm is easy, but god damn it also takes up some space..
- Ray tracing beyond
    - This was pretty damn fun to do, so maybe somewhere down the line I'll do Ray tracing in a week and or Ray tracing the rest of your life. But I think I'm more interested in the GPU, so we'll see where things go.
