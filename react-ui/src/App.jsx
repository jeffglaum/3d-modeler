import React, { useEffect, useRef} from 'react';
import init, { start_rendering } from 'rust-renderer'; // from WASM
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
      await init(); // initialize WASM module
      const canvas = canvasRef.current;
      if (canvas) {
        start_rendering(canvas);
      }
    };
    run();
  }, []);

  return (
    <div className="w-full p-4 bg-gray-100 flex justify-start">
      <DropdownAppBar />
      <canvas ref={canvasRef} width={1920} height={1080} />
    </div>
  );
}

export default App;

