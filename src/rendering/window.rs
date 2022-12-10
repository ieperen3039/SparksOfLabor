use core::time::Duration;

use glfw::Context;

extern crate glfw;

/**
 * A window which initializes GLFW and manages it.
 */
pub struct Window {
    window: glfw::Window,
    events : std::sync::mpsc::Receiver<(f64, glfw::WindowEvent)>,
    width: u32,
    height: u32,
    isFullScreen: bool,
    mouseIsCaptured: bool,
}

impl Window {
    pub fn new(width :u32, height: u32) -> Window {
        let glfwRef = glfw::init(glfw::FAIL_ON_ERRORS).expect("Could not intialize GLFW");

        // Create a windowed mode window and its OpenGL context
        let (mut window, events) = glfwRef.create_window(
            width, 
            height, 
            "Sparks Of Labor", 
            glfw::WindowMode::Windowed
        ).expect("Could not create GLFW window.");
        
        let mut result = Window {
            window,
            events,
            width,
            height,
            isFullScreen: false,
            mouseIsCaptured: false,
        };

        // Make the window's context current
        result.window.make_current();
        result.window.set_key_polling(true);

        
//         // Set clear color to black
//         glClearColor(0f, 0f, 0f, 0f); // black
// //        glClearColor(1f, 1f, 1f, 0f); // white

//         // Enable Depth Test
//         glEnable(GL_DEPTH_TEST);
//         // Enable Stencil Test
//         glEnable(GL_STENCIL_TEST);
//         // Support transparencies
//         glEnable(GL_BLEND);
//         glBlendFunc(GL_SRC_ALPHA, GL_ONE_MINUS_SRC_ALPHA);
//         if (settings.cullFace) {
//             // face culling
//             glEnable(GL_CULL_FACE);
//             glCullFace(GL_FRONT);
//         }

//         // set polygons to fill
//         glPolygonMode(GL_FRONT_AND_BACK, GL_FILL);


        return result;
    }

    pub fn update_until_closed(&self)
    {
        // Loop until the user closes the window
        while !self.window.should_close() {
            self.update();
            std::thread::sleep(Duration::from_millis(20));
        }
    }

    pub fn update(&self)
    {
        // Swap front and back buffers
        self.window.swap_buffers();

        // Poll for and process events
        self.window.glfw.poll_events();

        for (_, event) in glfw::flush_messages(&self.events) {
            Self::handle_window_event(event, &mut self.window);
        }
    }

    fn handle_window_event(event: glfw::WindowEvent, window: &mut glfw::Window) {
        println!("{:?}", event);
        
        match event {
            glfw::WindowEvent::Key(glfw::Key::Escape, _, glfw::Action::Press, _) => {
                window.set_should_close(true)
            }
            _ => {}
        }
    }
}


//     /**
//      * creates a window of the given width and height
//      * @param width  in pixels
//      * @param height in pixels
//      */
//     private long getWindow(int width, int height) {
//         // Create window
//         long newWindow = glfwCreateWindow(width, height, title, NULL, NULL);
//         if (newWindow == NULL) {
//             throw new RuntimeException("Failed to create the GLFW window");
//         }
//         this.width = width;
//         this.height = height;

// //        glfwSetWindowIcon(newWindow, null); // TODO icon

//         if (this.resizable) {
//             // Setup resize callback
//             glfwSetFramebufferSizeCallback(newWindow, (window, newWidth, newHeight) -> {
//                 this.width = newWidth;
//                 this.height = newHeight;
//                 sizeChangeListeners.forEach(l -> l.onChange(newWidth, newHeight));
//             });
//         }

//         // Make GL context current
//         glfwMakeContextCurrent(newWindow);
//         return newWindow;
//     }

//     /**
//      * update the {@link GLFWWindow}. This will deal with basic OpenGL formalities. Besides it will also poll for events
//      * which occurred on the window. Finally returns whether the window should close.
//      */
//     public void update() {
//         // Swap buffers
//         glfwSwapBuffers(window);

//         // Poll for events
//         glfwPollEvents();
//     }

//     /**
//      * saves a copy of the front buffer (the display) to disc
//      * @param dir          directory to store the image to
//      * @param filename     the file to save to
//      * @param bufferToRead the GL buffer to read, usually one of {@link GL11#GL_FRONT} or {@link GL11#GL_BACK}
//      */
//     public void printScreen(Directory dir, String filename, int bufferToRead) {
//         glReadBuffer(bufferToRead);
//         int bpp = 4; // Assuming a 32-bit display with a byte each for red, green, blue, and alpha.
//         ByteBuffer buffer = BufferUtils.createByteBuffer(width * height * bpp);
//         glReadPixels(0, 0, width, height, GL11.GL_RGBA, GL11.GL_UNSIGNED_BYTE, buffer);

//         new Thread(() ->
//                 Toolbox.writePNG(dir, filename, buffer, bpp, width, height),
//                 "Writing frame to disc"
//         ).start();
//     }

//     /**
//      * hints the window to close
//      */
//     public void close() {
//         glfwSetWindowShouldClose(window, true);
//     }

//     public void open() {
//         // Show window
//         glfwShowWindow(window);
//         glfwFocusWindow(window);
//     }

//     /**
//      * Terminate GLFW and release GLFW error callback
//      */
//     public void cleanup() {
//         sizeChangeListeners.clear();
//         glfwFreeCallbacks(window);
//         glfwDestroyWindow(window);
//         glfwTerminate();
//         glfwSetErrorCallback(null).free();
//     }


//     /**
//      * Set the color which is used for clearing the window.
//      * @param red   The red value (0.0 - 1.0)
//      * @param green The green value (0.0 - 1.0)
//      * @param blue  The blue value (0.0 - 1.0)
//      * @param alpha The alpha value (0.0 - 1.0)
//      */
//     public void setClearColor(float red, float green, float blue, float alpha) {
//         glClearColor(red, green, blue, alpha);
//     }

//     /**
//      * Check whether a certain key is pressed.
//      * @param keyCode The keycode of the key.
//      * @return Whether the key with requested keyCode is pressed.
//      */
//     public boolean isKeyPressed(int keyCode) {
//         return glfwGetKey(window, keyCode) == GLFW_PRESS;
//     }

//     /**
//      * Check whether a certain mouse button is pressed.
//      * @param button The button of the mouse.
//      * @return Whether the requested button is pressed.
//      */
//     public boolean isMouseButtonPressed(int button) {
//         return glfwGetMouseButton(window, button) == GLFW_PRESS;
//     }

//     /**
//      * Get the current position of the mouse.
//      * @return the position of the cursor, in screen coordinates, relative to the upper-left corner of the client area
//      * of the specified window
//      */
//     public Vector2i getMousePosition() {
//         glfwGetCursorPos(window, mousePosX, mousePosY);
//         return new Vector2i((int) mousePosX.get(0), (int) mousePosY.get(0));
//     }

//     /**
//      * Get whether the window should close.
//      * @return Whether the window should close.
//      */
//     public boolean shouldClose() {
//         return glfwWindowShouldClose(window);
//     }

//     /**
//      * Get the width of the window.
//      * @return The width of the window.
//      */
//     public int getWidth() {
//         return width;
//     }

//     /**
//      * Get the height of the window.
//      * @return The height of the window.
//      */
//     public int getHeight() {
//         return height;
//     }

//     /**
//      * Get whether resizing the window is allowed.
//      * @return Whether resizing the window is allowed.
//      */
//     public boolean resizeEnabled() {
//         return resizable;
//     }

//     public void setFullScreen(Settings settings) {
//         GLFWVidMode vidmode = glfwGetVideoMode(primaryMonitor);
//         glfwSetWindowMonitor(window, primaryMonitor, 0, 0, vidmode.width(), vidmode.height(), settings.targetFPS);

//         if (settings.vSync) {
//             // Turn on vSync
//             glfwSwapInterval(1);
//         }

//         fullScreen = true;
//     }

//     public void setWindowed(Settings settings) {
//         // Get primary display resolution
//         GLFWVidMode vidmode = glfwGetVideoMode(primaryMonitor);
//         // Center window on display
//         glfwSetWindowPos(
//                 window,
//                 (vidmode.width() - settings.windowWidth) / 2,
//                 (vidmode.height() - settings.windowHeight) / 2
//         );
//         fullScreen = false;
//     }

//     public void toggleFullScreen() {
//         if (fullScreen) {
//             setWindowed(settings);
//         } else {
//             setFullScreen(settings);
//         }
//     }

//     /** sets the mouse pointer to the given mode */
//     public void setCursorMode(CursorMode mode) {
//         switch (mode) {
//             case VISIBLE:
//                 glfwSetInputMode(window, GLFW_CURSOR, GLFW_CURSOR_NORMAL);
//                 break;
//             case HIDDEN_FREE:
//                 glfwSetInputMode(window, GLFW_CURSOR, GLFW_CURSOR_HIDDEN);
//                 break;
//             case HIDDEN_CAPTURED:
//                 glfwSetInputMode(window, GLFW_CURSOR, GLFW_CURSOR_DISABLED);
//                 break;
//         }
//     }

//     public void setClearColor(Color4f color4f) {
//         setClearColor(color4f.red, color4f.green, color4f.blue, color4f.alpha);
//     }

//     /**
//      * Sets the callbacks to the given listeners. The values that are null are skipped.
//      */
//     public void setCallbacks(
//             GLFWKeyCallbackI key, GLFWMouseButtonCallbackI mousePress, GLFWCursorPosCallbackI mouseMove,
//             GLFWScrollCallbackI mouseScroll
//     ) {
//         if (key != null) glfwSetKeyCallback(window, key);
//         if (mousePress != null) glfwSetMouseButtonCallback(window, mousePress);
//         if (mouseMove != null) glfwSetCursorPosCallback(window, mouseMove);
//         if (mouseScroll != null) glfwSetScrollCallback(window, mouseScroll);
//     }

//     public void setTextCallback(GLFWCharCallbackI input) {
//         glfwSetCharCallback(window, input);
//     }

//     public void addResizeListener(ResizeListener listener) {
//         sizeChangeListeners.add(listener);
//     }

//     public void setMinimized(boolean doMinimize) {
//         if (doMinimize) {
//             glfwHideWindow(window);
//         } else {
//             glfwShowWindow(window);
//         }
//     }

//     public Thread getOpenGLThread() {
//         return glContext;
//     }

//     public interface ResizeListener {
//         void onChange(int width, int height);
//     }

//     public static class Settings {
//         final boolean debugMode;
//         final boolean glDebugMessages;
//         final int antialiasLevel;
//         final int windowWidth;
//         final int windowHeight;
//         final boolean vSync;
//         final int targetFPS;
//         final boolean fullscreen;
//         final boolean cullFace;

//         public Settings() {
//             this(false, false, 1, false, 800, 600, false, 60, false);
//         }

//         public Settings(NG.Settings.Settings s) {
//             this(
//                     s.DEBUG, false,
//                     s.ANTIALIAS_LEVEL, !s.DEBUG,
//                     s.WINDOW_WIDTH, s.WINDOW_HEIGHT,
//                     s.V_SYNC, s.TARGET_FPS, true
//             );
//         }

//         public Settings(
//                 boolean debugMode, boolean glDebugMessages, int antialiasLevel, boolean fullscreen, int windowWidth,
//                 int windowHeight, boolean vSync, int targetFPS, boolean cullFace
//         ) {
//             this.debugMode = debugMode;
//             this.glDebugMessages = glDebugMessages;
//             this.antialiasLevel = antialiasLevel;
//             this.fullscreen = fullscreen;
//             this.windowWidth = windowWidth;
//             this.windowHeight = windowHeight;
//             this.vSync = vSync;
//             this.targetFPS = targetFPS;
//             this.cullFace = cullFace;
//         }
//     }
// }

// enum CursorMode {VISIBLE, HIDDEN_FREE, HIDDEN_CAPTURED}
