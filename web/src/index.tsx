import React from 'react';
import ReactDOM from 'react-dom';
import './index.css';
import App from './App';
import * as serviceWorker from './serviceWorker';
import { HashRouter } from 'react-router-dom';
import { initializeIcons } from '@fluentui/react';
import { ApplicationInsights } from '@microsoft/applicationinsights-web'

initializeIcons();

const appInsights = new ApplicationInsights({
  config: {
    instrumentationKey: '2235f5d2-318d-4abe-afef-b8ba3c73b7de'
  },
});
appInsights.loadAppInsights();
appInsights.trackPageView();

ReactDOM.render(
  // <React.StrictMode>
    <HashRouter>
      <App />
    </HashRouter>
  // </React.StrictMode>
  ,
  document.getElementById('root')
);

// If you want your app to work offline and load faster, you can change
// unregister() to register() below. Note this comes with some pitfalls.
// Learn more about service workers: https://bit.ly/CRA-PWA
serviceWorker.unregister();
