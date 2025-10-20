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

import React, { useState, useEffect } from "react";
import Layout from "../components/Layout";
import "./Settings.css";

function Settings() {
  const [settings, setSettings] = useState([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState("");
  const [saving, setSaving] = useState(false);
  const [showCreateForm, setShowCreateForm] = useState(false);
  const [newSetting, setNewSetting] = useState({
    key: "",
    value: "false",
    category: "system",
  });

  useEffect(() => {
    fetchSettings();
  }, []);

  async function fetchSettings() {
    try {
      setLoading(true);
      const response = await fetch("/api/admin/settings", {
        credentials: "include",
      });

      if (!response.ok) {
        throw new Error("Failed to fetch settings");
      }

      const data = await response.json();
      setSettings(data);
    } catch (err) {
      setError(err.message);
    } finally {
      setLoading(false);
    }
  }

  async function handleToggleSetting(setting) {
    setSaving(true);
    setError("");

    try {
      const newValue = setting.value === "true" ? "false" : "true";

      const response = await fetch("/api/admin/settings", {
        method: "PUT",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          key: setting.key,
          value: newValue,
          category: setting.category,
        }),
        credentials: "include",
      });

      if (!response.ok) {
        throw new Error("Failed to update setting");
      }

      setSettings(
        settings.map((s) =>
          s.id === setting.id ? { ...s, value: newValue } : s,
        ),
      );
    } catch (err) {
      setError(err.message);
    } finally {
      setSaving(false);
    }
  }

  function getSettingLabel(key) {
    const labels = {
      admin_registration_enabled: "Admin Registration",
    };
    return labels[key] || key;
  }

  function getSettingDescription(key) {
    const descriptions = {
      admin_registration_enabled:
        "Allow new administrators to register accounts via the registration page",
    };
    return descriptions[key] || "";
  }

  if (loading) {
    return <div className="loading">Loading settings...</div>;
  }

  return (
    <Layout>
      <div className="settings-page">
        <header className="page-header">
          <h1>System Settings</h1>
        </header>

        {error && <div className="error">{error}</div>}

        <div className="settings-list">
          {settings.length === 0 ? (
            <div className="empty-state">
              <p>No settings configured.</p>
            </div>
          ) : (
            settings.map((setting) => (
              <div key={setting.id} className="setting-item">
                <div className="setting-info">
                  <div className="setting-label">
                    {getSettingLabel(setting.key)}
                  </div>
                  {getSettingDescription(setting.key) && (
                    <div className="setting-description">
                      {getSettingDescription(setting.key)}
                    </div>
                  )}
                  {setting.category && (
                    <div className="setting-category">
                      <span className="badge">{setting.category}</span>
                    </div>
                  )}
                </div>
                <div className="setting-control">
                  <label className="toggle-switch">
                    <input
                      type="checkbox"
                      checked={setting.value === "true"}
                      onChange={() => handleToggleSetting(setting)}
                      disabled={saving}
                    />
                    <span className="toggle-slider"></span>
                  </label>
                  <span className="setting-value">
                    {setting.value === "true" ? "Enabled" : "Disabled"}
                  </span>
                </div>
              </div>
            ))
          )}
        </div>
      </div>
    </Layout>
  );
}

export default Settings;
