import { beforeEach, describe, expect, it } from 'vitest';
import { TestBed } from '@angular/core/testing';

import { sampleWithNewData, sampleWithRequiredData } from '../candidate-profile.test-samples';

import { CandidateProfileFormService } from './candidate-profile-form.service';

describe('CandidateProfile Form Service', () => {
  let service: CandidateProfileFormService;

  beforeEach(() => {
    service = TestBed.inject(CandidateProfileFormService);
  });

  describe('Service methods', () => {
    describe('createCandidateProfileFormGroup', () => {
      it('should create a new form with FormControl', () => {
        const formGroup = service.createCandidateProfileFormGroup();

        expect(formGroup.controls).toEqual(
          expect.objectContaining({
            id: expect.any(Object),
            userId: expect.any(Object),
            fullName: expect.any(Object),
            email: expect.any(Object),
            location: expect.any(Object),
            yearsOfExperience: expect.any(Object),
            skills: expect.any(Object),
            experiences: expect.any(Object),
            preferredRoles: expect.any(Object),
            languages: expect.any(Object),
            education: expect.any(Object),
            certifications: expect.any(Object),
            rawMarkdown: expect.any(Object),
            cvFilename: expect.any(Object),
            embeddingModel: expect.any(Object),
            embeddedAt: expect.any(Object),
            createdAt: expect.any(Object),
            updatedAt: expect.any(Object),
          }),
        );
      });

      it('passing ICandidateProfile should create a new form with FormGroup', () => {
        const formGroup = service.createCandidateProfileFormGroup(sampleWithRequiredData);

        expect(formGroup.controls).toEqual(
          expect.objectContaining({
            id: expect.any(Object),
            userId: expect.any(Object),
            fullName: expect.any(Object),
            email: expect.any(Object),
            location: expect.any(Object),
            yearsOfExperience: expect.any(Object),
            skills: expect.any(Object),
            experiences: expect.any(Object),
            preferredRoles: expect.any(Object),
            languages: expect.any(Object),
            education: expect.any(Object),
            certifications: expect.any(Object),
            rawMarkdown: expect.any(Object),
            cvFilename: expect.any(Object),
            embeddingModel: expect.any(Object),
            embeddedAt: expect.any(Object),
            createdAt: expect.any(Object),
            updatedAt: expect.any(Object),
          }),
        );
      });
    });

    describe('getCandidateProfile', () => {
      it('should return NewCandidateProfile for default CandidateProfile initial value', () => {
        const formGroup = service.createCandidateProfileFormGroup(sampleWithNewData);

        const candidateProfile = service.getCandidateProfile(formGroup);

        expect(candidateProfile).toMatchObject(sampleWithNewData);
      });

      it('should return NewCandidateProfile for empty CandidateProfile initial value', () => {
        const formGroup = service.createCandidateProfileFormGroup();

        const candidateProfile = service.getCandidateProfile(formGroup);

        expect(candidateProfile).toMatchObject({});
      });

      it('should return ICandidateProfile', () => {
        const formGroup = service.createCandidateProfileFormGroup(sampleWithRequiredData);

        const candidateProfile = service.getCandidateProfile(formGroup);

        expect(candidateProfile).toMatchObject(sampleWithRequiredData);
      });
    });

    describe('resetForm', () => {
      it('passing ICandidateProfile should not enable id FormControl', () => {
        const formGroup = service.createCandidateProfileFormGroup();
        expect(formGroup.controls.id.disabled).toBe(true);

        service.resetForm(formGroup, sampleWithRequiredData);

        expect(formGroup.controls.id.disabled).toBe(true);
      });

      it('passing NewCandidateProfile should disable id FormControl', () => {
        const formGroup = service.createCandidateProfileFormGroup(sampleWithRequiredData);
        expect(formGroup.controls.id.disabled).toBe(true);

        service.resetForm(formGroup, { id: null });

        expect(formGroup.controls.id.disabled).toBe(true);
      });
    });
  });
});
