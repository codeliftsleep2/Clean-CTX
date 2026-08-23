package com.example.demo;

import java.io.Serializable;
import java.time.LocalDateTime;

/**
 * Test fixture: Plain Java domain class for language layer tests.
 */
public abstract class BaseEntity implements Serializable {

    private Long id;
    private LocalDateTime createdAt;
    private LocalDateTime updatedAt;

    public BaseEntity() {
    }

    public Long getId() {
        return id;
    }

    public void setId(Long id) {
        this.id = id;
    }

    public LocalDateTime getCreatedAt() {
        return createdAt;
    }

    public void setCreatedAt(LocalDateTime createdAt) {
        this.createdAt = createdAt;
    }
}

class UserDto {
    private Long id;
    private String email;
    private String name;

    public UserDto() {
    }

    public Long getId() {
        return id;
    }

    public void setId(Long id) {
        this.id = id;
    }

    public String getEmail() {
        return email;
    }

    public void setEmail(String email) {
        this.email = email;
    }

    public String getName() {
        return name;
    }

    public void setName(String name) {
        this.name = name;
    }
}

record CreateUserRequest(String email, String name) {
}

record UpdateUserRequest(String email, String name) {
}

interface UserRepository {
    List<UserDto> findAll();
    UserDto findById(Long id);
    UserDto save(CreateUserRequest request);
    UserDto update(Long id, UpdateUserRequest request);
    void delete(Long id);
}