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
import "./AccessLogs.css";

function AccessLogs() {
  const [logs, setLogs] = useState([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState("");
  const [currentPage, setCurrentPage] = useState(1);
  const [totalPages, setTotalPages] = useState(1);
  const [total, setTotal] = useState(0);
  const [perPage, setPerPage] = useState(100);

  useEffect(() => {
    fetchLogs(currentPage);
  }, [currentPage]);

  async function fetchLogs(page = 1) {
    try {
      setLoading(true);
      const response = await fetch(
        `/api/admin/access-logs?page=${page}&per_page=${perPage}`,
        {
          credentials: "include",
        },
      );

      if (!response.ok) {
        throw new Error("Failed to fetch access logs");
      }

      const data = await response.json();
      setLogs(data.data);
      setTotal(data.total);
      setTotalPages(data.total_pages);
      setCurrentPage(data.page);
    } catch (err) {
      setError(err.message);
    } finally {
      setLoading(false);
    }
  }

  async function handleClearLogs() {
    if (
      !confirm(
        "Are you sure you want to clear all access logs? This action cannot be undone.",
      )
    ) {
      return;
    }

    try {
      const response = await fetch("/api/admin/access-logs", {
        method: "DELETE",
        credentials: "include",
      });

      if (!response.ok) {
        throw new Error("Failed to clear access logs");
      }

      // Reset to first page
      setCurrentPage(1);
      await fetchLogs(1);
    } catch (err) {
      setError(err.message);
    }
  }

  function handlePageChange(newPage) {
    if (newPage >= 1 && newPage <= totalPages) {
      setCurrentPage(newPage);
    }
  }

  function formatDate(dateString) {
    if (!dateString) return "N/A";
    const date = new Date(dateString);
    return date.toLocaleDateString() + " " + date.toLocaleTimeString();
  }

  function truncateUserAgent(userAgent) {
    if (!userAgent) return "N/A";
    return userAgent.length > 50
      ? userAgent.substring(0, 50) + "..."
      : userAgent;
  }

  if (loading) {
    return <div className="loading">Loading access logs...</div>;
  }

  return (
    <Layout>
      <div className="access-logs-page">
        <header className="page-header">
          <h1>Access Logs</h1>
          <button onClick={handleClearLogs} className="btn-danger">
            Clear All Logs
          </button>
        </header>

        {error && <div className="error">{error}</div>}

        <div className="logs-stats">
          <div className="stat-card">
            <div className="stat-label">Total Logs</div>
            <div className="stat-value">{total}</div>
          </div>
          <div className="stat-card">
            <div className="stat-label">Successful Accesses</div>
            <div className="stat-value">
              {logs.filter((log) => log.success).length}
            </div>
          </div>
          <div className="stat-card">
            <div className="stat-label">Failed Attempts</div>
            <div className="stat-value">
              {logs.filter((log) => !log.success).length}
            </div>
          </div>
          <div className="stat-card">
            <div className="stat-label">Current Page</div>
            <div className="stat-value">
              {currentPage} / {totalPages}
            </div>
          </div>
        </div>

        {logs.length === 0 ? (
          <div className="empty-state">
            <p>No access logs yet.</p>
          </div>
        ) : (
          <div className="logs-table-container">
            <table className="logs-table">
              <thead>
                <tr>
                  <th>Timestamp</th>
                  <th>Access Code</th>
                  <th>IP Address</th>
                  <th>Action</th>
                  <th>Status</th>
                  <th>User Agent</th>
                  <th>Count</th>
                </tr>
              </thead>
              <tbody>
                {logs.map((log) => (
                  <tr key={log.id} className={log.success ? "" : "failed"}>
                    <td>{formatDate(log.created_at)}</td>
                    <td>
                      <code>{log.access_code}</code>
                    </td>
                    <td>{log.ip_address || "N/A"}</td>
                    <td>
                      <span
                        className={`badge badge-method badge-${log.action.toLowerCase()}`}
                      >
                        {log.action}
                      </span>
                    </td>
                    <td>
                      <span
                        className={`badge ${log.success ? "badge-success" : "badge-failed"}`}
                      >
                        {log.success ? "Success" : "Failed"}
                      </span>
                    </td>
                    <td title={log.user_agent}>
                      {truncateUserAgent(log.user_agent)}
                    </td>
                    <td>{log.count || 0}</td>
                  </tr>
                ))}
              </tbody>
            </table>

            {totalPages > 1 && (
              <div className="pagination">
                <button
                  onClick={() => handlePageChange(1)}
                  disabled={currentPage === 1}
                  className="pagination-btn"
                >
                  First
                </button>
                <button
                  onClick={() => handlePageChange(currentPage - 1)}
                  disabled={currentPage === 1}
                  className="pagination-btn"
                >
                  Previous
                </button>
                <span className="pagination-info">
                  Page {currentPage} of {totalPages}
                </span>
                <button
                  onClick={() => handlePageChange(currentPage + 1)}
                  disabled={currentPage === totalPages}
                  className="pagination-btn"
                >
                  Next
                </button>
                <button
                  onClick={() => handlePageChange(totalPages)}
                  disabled={currentPage === totalPages}
                  className="pagination-btn"
                >
                  Last
                </button>
              </div>
            )}
          </div>
        )}
      </div>
    </Layout>
  );
}

export default AccessLogs;
