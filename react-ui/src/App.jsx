import React, { useEffect, useRef } from 'react';
//import init, { start_rendering, handle_mouse_click } from 'rust-renderer'; // from WASM
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
      console.log("Selected file:", file.name);
      const reader = new FileReader();
      reader.onload = (e) => {
        console.log("File content:", e.target.result);
      };
      reader.readAsText(file);
    }
  };

  return (
        <>
    <AppBar position="static">
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
          <MenuItem onClick={handleFileMenuClose}>Option 2</MenuItem>
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
    const run = async () => {
      const canvas = canvasRef.current;
      const wasmModule = await init();

      window.wasm = wasm;

      if (canvas) {
        wasm.start_rendering(canvas);
      }
    };
    run();
  }, []);

  const handleClick = (e) => {
    const rect = canvasRef.current.getBoundingClientRect();
    const x = e.clientX - rect.left;
    const y = e.clientY - rect.top;

    if (window.wasm && window.wasm.handle_mouse_click) {
      window.wasm.handle_mouse_click(x, y);
    }
  };

  return (
    <div className="w-full p-4 bg-gray-100 flex justify-start">
      <DropdownAppBar />
      <canvas ref={canvasRef} width={1920} height={1080} onClick={handleClick}/>
    </div>
  );
}

export default App;

