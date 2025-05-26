import React, { useEffect, useRef } from 'react';
import init, * as wasm from 'rust-renderer';
import { AppBar, Toolbar, Button, Menu, MenuItem } from '@mui/material';

function DropdownAppBar() {

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
      const reader = new FileReader();
      reader.onload = (e) => {
        const fileContent = e.target.result;

        if (window.wasm && window.wasm.process_file_content) {
          window.wasm.process_file_content(fileContent);
        } else {
          console.error("Rust WebAssembly function not found!");
        }
      };
      reader.readAsText(file);
    }
  };

  const handleToggleWireframe = () => {
    if (window.wasm && window.wasm.toggle_wireframe) {
      window.wasm.toggle_wireframe();
    } else {
      console.error("Rust WebAssembly function 'toggle_wireframe' not found!");
    }
    handleDrawMenuClose();
  };

  return (
        <>
    <AppBar position="sticky" style={{width: 'calc(100vw - 16px)', backgroundColor: '#262626', margin: 0, padding: 0, boxSizing: 'border-box'}}>
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
          <MenuItem onClick={handleToggleWireframe}>Toggle Wireframe</MenuItem>
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

    resizeCanvas();

    window.addEventListener('resize', resizeCanvas);

    const run = async () => {
      const wasmModule = await init();
      window.wasm = wasm;

      if (canvas) {
        wasm.main(canvas);
      }
    };
    run();

    // cleanup event listener on unmount
    return () => {
      document.body.style.overflow = '';
      document.documentElement.style.overflow = '';
      window.removeEventListener('resize', resizeCanvas);
    };
  }, []);

  return (
    <div style={{height: 'calc(100vh - 16px)'}}>
      <DropdownAppBar />
      <canvas ref={canvasRef} />
    </div>
  );
}

export default App;
