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
import "./AccessCodes.css";

function AccessCodes() {
  const [codes, setCodes] = useState([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState("");
  const [showCreateForm, setShowCreateForm] = useState(false);
  const [newCode, setNewCode] = useState({
    code: "",
    name: "",
    expires_at: "",
  });

  useEffect(() => {
    fetchCodes();
  }, []);

  function generateRandomCode() {
    // Generate a secure random code using crypto API
    // Format: 12 characters of base62 (alphanumeric, case-sensitive)
    const array = new Uint8Array(12);
    crypto.getRandomValues(array);

    // Convert to base62 (0-9, a-z, A-Z)
    const base62 =
      "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
    let code = "";
    for (let i = 0; i < array.length; i++) {
      code += base62[array[i] % base62.length];
    }

    return code;
  }

  function handleGenerateCode() {
    setNewCode({ ...newCode, code: generateRandomCode() });
  }

  async function fetchCodes() {
    try {
      const response = await fetch("/api/admin/access-codes", {
        credentials: "include",
      });

      if (!response.ok) {
        throw new Error("Failed to fetch access codes");
      }

      const data = await response.json();
      setCodes(data);
    } catch (err) {
      setError(err.message);
    } finally {
      setLoading(false);
    }
  }

  async function handleCreateCode(e) {
    e.preventDefault();
    setError("");

    try {
      const payload = {
        code: newCode.code,
        name: newCode.name,
        expires_at: newCode.expires_at || null,
      };

      const response = await fetch("/api/admin/access-codes", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(payload),
        credentials: "include",
      });

      if (!response.ok) {
        const data = await response.json();
        throw new Error(data.error || "Failed to create access code");
      }

      // Reset form and refresh list
      setNewCode({ code: "", name: "", expires_at: "" });
      setShowCreateForm(false);
      await fetchCodes();
    } catch (err) {
      setError(err.message);
    }
  }

  async function handleDeleteCode(id) {
    if (!confirm("Are you sure you want to delete this access code?")) {
      return;
    }

    try {
      const response = await fetch(`/api/admin/access-codes/${id}`, {
        method: "DELETE",
        credentials: "include",
      });

      if (!response.ok) {
        throw new Error("Failed to delete access code");
      }

      await fetchCodes();
    } catch (err) {
      setError(err.message);
    }
  }

  function formatDate(dateString) {
    if (!dateString) return "Never";
    const date = new Date(dateString);
    return date.toLocaleDateString() + " " + date.toLocaleTimeString();
  }

  if (loading) {
    return <div className="loading">Loading access codes...</div>;
  }

  return (
    <Layout>
      <div className="access-codes-page">
        <header className="page-header">
          <h1>Access Code Management</h1>
          <button
            onClick={() => setShowCreateForm(!showCreateForm)}
            className="btn-primary"
          >
            {showCreateForm ? "Cancel" : "+ New Access Code"}
          </button>
        </header>

        {error && <div className="error">{error}</div>}

        {showCreateForm && (
          <div className="create-form-container">
            <form onSubmit={handleCreateCode} className="create-form">
              <h2>Create New Access Code</h2>

              <div className="form-group">
                <label htmlFor="code">Access Code *</label>
                <div className="input-with-button">
                  <input
                    type="text"
                    id="code"
                    value={newCode.code}
                    onChange={(e) =>
                      setNewCode({ ...newCode, code: e.target.value })
                    }
                    required
                    placeholder="e.g., resume-2025"
                  />
                  <button
                    type="button"
                    onClick={handleGenerateCode}
                    className="btn-generate"
                  >
                    Generate
                  </button>
                </div>
              </div>

              <div className="form-group">
                <label htmlFor="name">Name/Description *</label>
                <input
                  type="text"
                  id="name"
                  value={newCode.name}
                  onChange={(e) =>
                    setNewCode({ ...newCode, name: e.target.value })
                  }
                  required
                  placeholder="e.g., Personal Link"
                />
              </div>

              <div className="form-group">
                <label htmlFor="expires_at">Expiration Date (Optional)</label>
                <input
                  type="datetime-local"
                  id="expires_at"
                  value={newCode.expires_at}
                  onChange={(e) =>
                    setNewCode({ ...newCode, expires_at: e.target.value })
                  }
                />
                <small>Leave empty for no expiration</small>
              </div>

              <div className="form-actions">
                <button type="submit" className="btn-primary">
                  Create Code
                </button>
                <button
                  type="button"
                  onClick={() => {
                    setShowCreateForm(false);
                    setNewCode({ code: "", name: "", expires_at: "" });
                  }}
                  className="btn-secondary"
                >
                  Cancel
                </button>
              </div>
            </form>
          </div>
        )}

        <div className="codes-list">
          <h2>Active Access Codes ({codes.length})</h2>

          {codes.length === 0 ? (
            <div className="empty-state">
              <p>No access codes yet. Create one to get started!</p>
            </div>
          ) : (
            <div className="codes-grid">
              {codes.map((code) => (
                <div
                  key={code.id}
                  className={`code-card ${code.is_expired ? "expired" : ""}`}
                >
                  <div className="code-header">
                    <h3>{code.name}</h3>
                    {code.is_expired && (
                      <span className="badge-expired">Expired</span>
                    )}
                  </div>

                  <div className="code-details">
                    <div className="code-value">
                      <strong>Code:</strong>
                      <code>{code.code}</code>
                    </div>

                    <div className="code-meta">
                      <div>
                        <strong>Expires:</strong> {formatDate(code.expires_at)}
                      </div>
                      <div>
                        <strong>Created:</strong> {formatDate(code.created_at)}
                      </div>
                      <div>
                        <strong>Usage Count:</strong> {code.usage_count || 0}
                      </div>
                    </div>
                  </div>

                  <div className="code-actions">
                    <button
                      onClick={() => handleDeleteCode(code.id)}
                      className="btn-delete"
                    >
                      Delete
                    </button>
                  </div>
                </div>
              ))}
            </div>
          )}
        </div>
      </div>
    </Layout>
  );
}

export default AccessCodes;
