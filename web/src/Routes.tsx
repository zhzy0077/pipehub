import React from 'react';
import {
    Switch,
    Route,
} from "react-router-dom";
import Home from './Home';
import User from './User';

function Routes() {
    return (
        <Switch>
            <Route path="/user">
                <User />
            </Route>
            <Route path="/">
                <Home />
            </Route>
        </Switch>
    );
}

export default Routes;
