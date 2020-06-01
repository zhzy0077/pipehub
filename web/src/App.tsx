import React from 'react';
import Routes from './Routes';
import Navs from './Navs';
import { Stack, Separator } from '@fluentui/react';

function App() {
  return (
    <Stack>
      <Navs />
      <Separator />
      <Routes />
    </Stack>
  );
}

export default App;
