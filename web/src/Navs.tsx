import React, { CSSProperties } from 'react';
import { Stack, FontSizes, Icon, Text } from '@fluentui/react';
import { Link } from 'react-router-dom';

const noLinkStyles: CSSProperties = { textDecoration: 'none', color: 'inherit' };

function Navs() {
    return (
        <Stack horizontal horizontalAlign="space-between">
            <Stack.Item>
                <Link style={noLinkStyles} to="/">
                    <Icon style={{ fontSize: FontSizes.xxLarge }} iconName="Robot" />
                    <Text variant='xxLarge' style={{ color: 'purple' }}>PipeHub</Text>
                </Link>
            </Stack.Item>
            <Stack.Item align='end'>
                <Link style={noLinkStyles} to="/user">
                    <Text variant='large' style={{ color: 'teal' }}>User</Text>
                </Link>
            </Stack.Item>
        </Stack>
    );
}

export default Navs;
