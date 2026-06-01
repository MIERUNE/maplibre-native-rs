#include "sources.h"
#include "geojson/geojson.h"
#include <mbgl/style/source.hpp>
#include <mbgl/style/sources/geojson_source.hpp>
#include <memory>

namespace mln::bridge::style::sources {
    std::unique_ptr<mbgl::style::Source> geojson_into_source(
        std::unique_ptr<mbgl::style::GeoJSONSource> source) {
        return source;
    }

    const mbgl::style::GeoJSONSource* source_as_geojson(const mbgl::style::Source* source) {
        if (!source) {
            return nullptr;
        }
        return source->as<mbgl::style::GeoJSONSource>();
    }

    mbgl::style::GeoJSONSource* source_as_geojson_mut(mbgl::style::Source* source) {
        if (!source) {
            return nullptr;
        }
        return source->as<mbgl::style::GeoJSONSource>();
    }
}

namespace mln::bridge::style::sources::geojson {
    std::unique_ptr<mbgl::style::GeoJSONSource> create(rust::Str id) {
        return std::make_unique<mbgl::style::GeoJSONSource>(std::string(id));
    }

    void setGeoJson(const std::unique_ptr<mbgl::style::GeoJSONSource>& source,
                    const mln::bridge::geojson::GeoJson& geojson) {
        source->setGeoJSON(geojson.get());
    }

    void setGeoJsonPtr(mbgl::style::GeoJSONSource* source,
                       const mln::bridge::geojson::GeoJson& geojson) {
        if (!source) {
            return;
        }
        source->setGeoJSON(geojson.get());
    }
}
