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
import { useNavigate } from "react-router-dom";
import Layout from "../components/Layout";
import "./Dashboard.css";

function Dashboard() {
  const navigate = useNavigate();

  return (
    <Layout>
      <div className="dashboard-content">
        <div className="welcome-card">
          <h2>Welcome to the Admin Panel</h2>
          <p>
            This is where you'll manage access codes and other administrative
            features.
          </p>
        </div>

        <div className="feature-grid">
          <div className="feature-card">
            <h3>Access Codes</h3>
            <p>Manage and edit access codes for the resume site.</p>
            <button
              className="btn-feature"
              onClick={() => navigate("/access-codes")}
            >
              Manage Codes
            </button>
          </div>

          <div className="feature-card">
            <h3>Access Logs</h3>
            <p>View access logs and usage statistics.</p>
            <button
              className="btn-feature"
              onClick={() => navigate("/access-logs")}
            >
              View Logs
            </button>
          </div>

          <div className="feature-card">
            <h3>Settings</h3>
            <p>Configure site settings and preferences.</p>
            <button
              className="btn-feature"
              onClick={() => navigate("/settings")}
            >
              Manage Settings
            </button>
          </div>
        </div>
      </div>
    </Layout>
  );
}

export default Dashboard;
