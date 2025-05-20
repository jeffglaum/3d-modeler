import React, { useEffect, useRef } from 'react';
//import init, { start_rendering, handle_mouse_click } from 'rust-renderer'; // from WASM
import init, * as wasm from 'rust-renderer';
import { AppBar, Toolbar, Button, Menu, MenuItem } from '@mui/material';

function DropdownAppBar({ canvasRef}) {

  const fileInputRef = useRef(null);
  const [fileMenuAnchorEl, setFileMenuAnchorEl] = React.useState(null);
  const [drawMenuAnchorEl, setDrawMenuAnchorEl] = React.useState(null);

  const handleFileMenuClick = (event) => {
    setFileMenuAnchorEl(event.currentTarget);
  };

  const handleFileMenuClose = () => {
    setFileMenuAnchorEl(null);
  };

  const handleDrawMenuClick = (event) => {
    setDrawMenuAnchorEl(event.currentTarget);
  };

  const handleDrawMenuClose = () => {
    setDrawMenuAnchorEl(null);
  };

  const handleFileOpen = () => {
    fileInputRef.current.click();
    handleFileMenuClose();
  };

  const handleFileChange = (event) => {
    const file = event.target.files[0];
    if (file) {
      //console.log("Selected file:", file.name);
      const reader = new FileReader();
      reader.onload = (e) => {
        const fileContent = e.target.result; // File content as a string
        //console.log("File content:", fileContent);

        // Call the Rust WebAssembly function
        if (window.wasm && window.wasm.process_file_content) {
          const gl = canvasRef.current.getContext('webgl2');
          window.wasm.process_file_content(fileContent, gl);
        } else {
          console.error("Rust WebAssembly function not found!");
        }
      };
      reader.readAsText(file); // Read the file as text
    }
  };

  const handleToggleWireframe = () => {
    if (window.wasm && window.wasm.toggle_wireframe) {
      window.wasm.toggle_wireframe(); // Call the Rust WASM function
    } else {
      console.error("Rust WebAssembly function 'toggle_wireframe' not found!");
    }
    handleFileMenuClose();
  };

  return (
        <>
    <AppBar position="fixed" style={{width: '100%'}}>
      <Toolbar>
        <Button
          color="inherit"
          onClick={handleFileMenuClick}
        >
          File
        </Button>
        <Menu
          anchorEl={fileMenuAnchorEl}
          open={Boolean(fileMenuAnchorEl)}
          onClose={handleFileMenuClose}
        >
          <MenuItem onClick={handleFileOpen}>Open</MenuItem>
          <MenuItem onClick={handleToggleWireframe}>Toggle Wireframe</MenuItem>
          <MenuItem onClick={handleFileMenuClose}>Option 3</MenuItem>
        </Menu>
        <Button
          color="inherit"
          onClick={handleDrawMenuClick}
        >
          Draw
        </Button>
        <Menu
          anchorEl={drawMenuAnchorEl}
          open={Boolean(drawMenuAnchorEl)}
          onClose={handleDrawMenuClose}
        >
          <MenuItem onClick={handleDrawMenuClose}>Option 1</MenuItem>
          <MenuItem onClick={handleDrawMenuClose}>Option 2</MenuItem>
        </Menu>
      </Toolbar>
    </AppBar>

    {/* Hidden file input */}
    <input
      type="file"
      ref={fileInputRef}
      style={{ display: 'none' }}
      onChange={handleFileChange}
    />
    </>
  );
}
function App() {
  const canvasRef = useRef(null);

  useEffect(() => {
    document.body.style.overflow = 'hidden';
    document.documentElement.style.overflow = 'hidden';
    
    const canvas = canvasRef.current;

    const resizeCanvas = () => {
      if (canvas) {
        const windowBorder = 15;
        canvas.width = window.innerWidth;
        canvas.height = window.innerHeight - windowBorder;
      }
    };

    // Set initial canvas size
    resizeCanvas();

    // Listen for window resize events
    window.addEventListener('resize', resizeCanvas);

    const run = async () => {
      const wasmModule = await init();
      window.wasm = wasm;

      if (canvas) {
        wasm.start_rendering(canvas);
      }
    };
    run();

    // Cleanup event listener on unmount
    return () => {
      document.body.style.overflow = '';
      document.documentElement.style.overflow = '';
      window.removeEventListener('resize', resizeCanvas);
    };
  }, []);

  const handleClick = (e) => {
    const rect = canvasRef.current.getBoundingClientRect();
    const x = e.clientX - rect.left;
    const y = e.clientY - rect.top;

    const canvas = canvasRef.current;
    if (canvas) {
      wasm.start_rendering(canvas);
    }

    if (window.wasm && window.wasm.handle_mouse_click) {
      window.wasm.handle_mouse_click(x, y);
    }
  };

  return (
    <div
      className="w-full bg-gray-100 flex justify-start"
      style={{ margin: 0, padding: 0, height: '100vh', width: '100vw' }}
    >
      <DropdownAppBar canvasRef={canvasRef} />
      <canvas ref={canvasRef} />
    </div>
  );
}

export default App;
