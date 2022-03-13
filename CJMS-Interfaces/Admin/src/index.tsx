import ReactDOM from 'react-dom';
import React, {useState, useEffect} from 'react';
import {Login, CJMS_Application, useToken, comm_service } from '@cjms_interfaces/shared';


function App() {
  comm_service.senders.sendClockTimeEvent(5);
  const { token, setToken } = useToken();
  if (!token) {
    console.log("Token not made, redirecting...");
    return <Login setToken={setToken} allowedUser={'admin'}/>;
  } else {
    console.log("Token made");
    return (
      <div>
        <h1>This is a generic page</h1>
      </div>
    );
  }
}

CJMS_Application(App);