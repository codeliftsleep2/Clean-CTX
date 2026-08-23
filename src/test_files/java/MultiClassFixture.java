package com.example.multiclass;

import org.springframework.stereotype.Service;
import org.springframework.web.bind.annotation.RestController;
import org.springframework.web.bind.annotation.RequestMapping;
import org.springframework.web.bind.annotation.GetMapping;
import org.springframework.web.bind.annotation.PutMapping;
import org.springframework.web.context.annotation.Bean;
import org.springframework.context.annotation.Configuration;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Value;
import java.io.Serializable;
import java.util.List;

/**
 * Multi-class regression fixture.
 *
 * Tests the per-class metadata invariant:
 *   A meta-layer may inspect only the exact source span belonging to the
 *   type it is enriching. It must never infer ownership from neighboring
 *   or whole-file text.
 *
 * This file contains multiple classes with different Spring annotations.
 * Each class's markers must be emitted ONLY for that class, never for
 * another class. The classes are deliberately interleaved with markers
 * that would be easy to cross-contaminate.
 */

// ════════════════════════════════════════════════════════════════════════
// Class 1: @RestController with GET /api/items
// Expected markers: Φrest:ItemController map=[/api/items,GET /{id},PUT /{id}]
// ════════════════════════════════════════════════════════════════════════
@RestController
@RequestMapping("/api/items")
public class ItemController {

    @Autowired
    private ItemService itemService;

    @GetMapping("/{id}")
    public Item getItem(@PathVariable Long id) {
        return itemService.findById(id);
    }

    @PutMapping("/{id}")
    public Item updateItem(@PathVariable Long id, @RequestBody Item item) {
        return itemService.update(id, item);
    }
}

// ════════════════════════════════════════════════════════════════════════
// Class 2: Plain POJO — NO Spring annotations
// Expected: NO Φ markers (not even Φsvc, Φrest, etc.)
// ════════════════════════════════════════════════════════════════════════
class Item {
    private Long id;
    private String name;
    private double price;

    public Item() {}
    public Long getId() { return id; }
    public void setId(Long id) { this.id = id; }
    public String getName() { return name; }
    public void setName(String n) { this.name = n; }
    public double getPrice() { return price; }
    public void setPrice(double p) { this.price = p; }
}

// ════════════════════════════════════════════════════════════════════════
// Class 3: @Service — should get Φsvc:ItemService
// CRITICAL: Must NOT inherit ItemController's @RestController markers
// ════════════════════════════════════════════════════════════════════════
@Service
public class ItemService {
    public Item findById(Long id) { return null; }
    public Item update(Long id, Item item) { return null; }
}

// ════════════════════════════════════════════════════════════════════════
// Class 4: @Configuration with @Bean — should get Φconf:AppConfig + Φbean:*
// CRITICAL: Must NOT inherit ItemController's @GetMapping or @RequestMapping
// ════════════════════════════════════════════════════════════════════════
@Configuration
public class AppConfig {
    @Bean
    public ItemService itemService() { return new ItemService(); }
}

// ════════════════════════════════════════════════════════════════════════
// Class 5: @RestController with DIFFERENT mapping — should get Φrest:HealthController
// CRITICAL: Must NOT include ItemController's request mappings
// ════════════════════════════════════════════════════════════════════════
@RestController
@RequestMapping("/health")
public class HealthController {

    @GetMapping
    public String health() { return "OK"; }
}

// ════════════════════════════════════════════════════════════════════════
// Class 6: Plain POJO — NO Spring annotations
// Expected: NO Φ markers
// ════════════════════════════════════════════════════════════════════════
class HealthResponse {
    private String status;
    private long timestamp;
    public String getStatus() { return status; }
    public void setStatus(String s) { this.status = s; }
}