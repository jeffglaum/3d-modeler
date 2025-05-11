import React, { useEffect, useRef } from 'react';
//import init, { start_rendering, handle_mouse_click } from 'rust-renderer'; // from WASM
import init, * as wasm from 'rust-renderer';
import { AppBar, Toolbar, Button, Menu, MenuItem } from '@mui/material';

function DropdownAppBar() {

  const [anchorEl, setAnchorEl] = React.useState(null);

  const handleClick = (event) => {
    setAnchorEl(event.currentTarget);
  };

  const handleClose = () => {
    setAnchorEl(null);
  };

  return (
    <AppBar position="static">
      <Toolbar>
        <Button
          color="inherit"
          onClick={handleClick}
        >
          File
        </Button>
        <Menu
          anchorEl={anchorEl}
          open={Boolean(anchorEl)}
          onClose={handleClose}
        >
          <MenuItem onClick={handleClose}>Option 1</MenuItem>
          <MenuItem onClick={handleClose}>Option 2</MenuItem>
          <MenuItem onClick={handleClose}>Option 3</MenuItem>
        </Menu>
        <Button
          color="inherit"
          onClick={handleClick}
        >
          Draw
        </Button>
        <Menu
          anchorEl={anchorEl}
          open={Boolean(anchorEl)}
          onClose={handleClose}
        >
          <MenuItem onClick={handleClose}>Option 1</MenuItem>
          <MenuItem onClick={handleClose}>Option 2</MenuItem>
          <MenuItem onClick={handleClose}>Option 3</MenuItem>
        </Menu>
      </Toolbar>
    </AppBar>
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

