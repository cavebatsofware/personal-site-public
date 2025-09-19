function initiateDownload() {
    // Extract the code from current URL path
    const pathParts = window.location.pathname.split('/');
    const code = pathParts[pathParts.length - 1]; // Get the code from /resume/{code}

    // Create a download link using the same protected route
    const link = document.createElement('a');
    link.href = `/document/${code}/download`; // Protected PDF download route
    link.download = 'Document.pdf';
    link.style.display = 'none';
    document.body.appendChild(link);

    setTimeout(() => {
        link.click();
        document.body.removeChild(link);
    }, 800);
}