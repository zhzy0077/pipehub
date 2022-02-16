import React from 'react';
import {
    Route,
    Routes
} from "react-router-dom";
import Home from './Home';
import User from './User';

function AppRoute() {
    return (
        <Routes>
            <Route path="/user" element={<User />} />
            <Route path="/" element={<Home />} />
        </Routes>
    );
}

export default AppRoute;
