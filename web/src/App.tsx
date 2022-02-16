import React from 'react';
import AppRoute from './Routes';
import Navs from './Navs';
import { Stack, Separator } from '@fluentui/react';

function App() {
  return (
    <Stack>
      <Navs />
      <Separator />
      <AppRoute />
    </Stack>
  );
}

export default App;
