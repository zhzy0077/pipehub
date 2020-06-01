import React from 'react';
import { Stack } from '@fluentui/react';
import GetStarted from './GetStarted';
import Faq from './Faq';

function Home() {
    return (
        <Stack tokens={{ childrenGap: '20px' }}>
            <Stack.Item><GetStarted /></Stack.Item>
            <Stack.Item><Faq /></Stack.Item>
        </Stack>
    );
}

export default Home;
