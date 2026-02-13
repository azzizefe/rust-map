# GeoPic Project Roadmap 🗺️

Dedicated to providing high-precision location finding from photos, optimized for Linux environments.

## Phase 1: Foundation (Current)
- [ ] **EXIF GPS Extraction**: Robust parsing of GPS metadata from various image formats (JPEG, TIFF, PNG).
- [ ] **Precision Conversion**: Converting Degrees/Minutes/Seconds (DMS) to Decimal Degrees with high accuracy.
- [ ] **Reverse Geocoding**: Integration with OpenStreetMap (Nominatim) for exact street-level address retrieval.
- [ ] **CLI Interface**: Clean Linux command-line interface with colored output.

## Phase 2: Enhanced Intelligence
- [ ] **Multi-API Support**: Support for Google Maps and Mapbox as fallback providers.
- [ ] **Batch Processing**: Ability to process entire directories of images and export results to JSON/CSV.
- [ ] **Map Visualization**: Generate static map images or HTML files showing the photo location.

## Phase 3: AI-Powered Location (Visual Search)
- [ ] **Landmark Recognition**: Use AI/ML models to identify famous landmarks even when GPS data is missing.
- [ ] **Environment Analysis**: Detect vegetation, architectural styles, and signage to estimate location.
- [ ] **Neural Network Integration**: Integrate with open-source visual geolocation models.

## Phase 4: User Interface & Exports
- [ ] **GUI for Linux**: A native GTK or Druid-based graphical interface.
- [ ] **KML/GPX Export**: Export location data for use in Google Earth and GPS devices.
