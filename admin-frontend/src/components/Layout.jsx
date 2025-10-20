/*  This file is part of a personal website project codename personal-site
 *  Copyright (C) 2025  Grant DeFayette
 *
 *  personal-site is free software: you can redistribute it and/or modify
 *  it under the terms of the GNU General Public License as published by
 *  the Free Software Foundation, either version 3 of the License, or
 *  (at your option) any later version.
 *
 *  personal-site is distributed in the hope that it will be useful,
 *  but WITHOUT ANY WARRANTY; without even the implied warranty of
 *  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 *  GNU General Public License for more details.
 *
 *  You should have received a copy of the GNU General Public License
 *  along with personal-site.  If not, see <https://www.gnu.org/licenses/>.
 */

import React from "react";
import { useNavigate, useLocation } from "react-router-dom";
import { useAuth } from "../contexts/AuthContext";
import "./Layout.css";

function Layout({ children }) {
  const { user, logout } = useAuth();
  const navigate = useNavigate();
  const location = useLocation();

  const isActive = (path) => {
    return location.pathname === path;
  };

  return (
    <div className="layout">
      <header className="layout-header">
        <div className="header-content">
          <div className="header-left">
            <h1 onClick={() => navigate("/dashboard")} className="site-title">
              Admin Dashboard
            </h1>
            <nav className="header-nav">
              <button
                className={`nav-link ${isActive("/dashboard") ? "active" : ""}`}
                onClick={() => navigate("/dashboard")}
              >
                Dashboard
              </button>
              <button
                className={`nav-link ${isActive("/access-codes") ? "active" : ""}`}
                onClick={() => navigate("/access-codes")}
              >
                Access Codes
              </button>
              <button
                className={`nav-link ${isActive("/access-logs") ? "active" : ""}`}
                onClick={() => navigate("/access-logs")}
              >
                Access Logs
              </button>
              <button
                className={`nav-link ${isActive("/settings") ? "active" : ""}`}
                onClick={() => navigate("/settings")}
              >
                Settings
              </button>
            </nav>
          </div>
          <div className="header-right">
            <span className="user-email">{user?.email}</span>
            <button onClick={logout} className="btn-logout">
              Logout
            </button>
          </div>
        </div>
      </header>

      <main className="layout-main">{children}</main>
    </div>
  );
}

export default Layout;
